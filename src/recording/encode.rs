//! The encoding half of a recording: pictures in, an H.264 video track in an MP4 file out.
//!
//! OpenH264 hands back each frame as NAL units in "Annex B" form (each behind a start code); an MP4
//! file wants them length-prefixed ("AVCC"), and the sequence and picture parameter sets (SPS, PPS)
//! not in the samples but in the track's header. So the track is added when the first frame (always
//! a keyframe) has given those two, and every frame's other NAL units become one sample.

use openh264::encoder::{BitRate, Encoder, EncoderConfig, FrameRate, FrameType, IntraFramePeriod};
use openh264::formats::{RgbaSliceU8, YUVBuffer};
use openh264::OpenH264API;
use std::io::{BufWriter, Write};

/// The MP4 track's clock: ticks per second.
const TIMESCALE: u32 = 90_000;

/// The size of the video for pictures of `size` (pixels): scaled down by the smallest whole factor
/// that fits `max_side`, then trimmed to even numbers (H.264 works in 2×2 blocks of colour).
pub(super) fn video_size(size: [usize; 2], max_side: u32) -> (u32, u32) {
    let longest = size[0].max(size[1]).max(1) as u32;
    let factor = longest.div_ceil(max_side).max(1);
    let even = |v: usize| ((v as u32 / factor) & !1).max(2);
    (even(size[0]), even(size[1]))
}

/// A video being written.
pub(super) struct Video {
    encoder: Encoder,
    mp4: Option<mp4::Mp4Writer<BufWriter<std::fs::File>>>,
    /// Held until the first frame gives the parameter sets the track header needs.
    file: Option<BufWriter<std::fs::File>>,
    width: u32,
    height: u32,
    fps: u32,
    frames: u64,
}

impl Video {
    pub(super) fn new(file: BufWriter<std::fs::File>, width: u32, height: u32, fps: u32) -> Result<Video, String> {
        // about a tenth of a bit per pixel: screen content (flat areas, sharp edges) stays crisp
        let bps = (width as u64 * height as u64 * fps as u64 / 10).clamp(1_500_000, 20_000_000) as u32;
        // start from the encoder's defaults (a known-good set) and change only what the video needs:
        // its frame rate, bit rate and a keyframe every two seconds so a player can seek
        let config = EncoderConfig::new()
            .bitrate(BitRate::from_bps(bps))
            .max_frame_rate(FrameRate::from_hz(fps as f32))
            .intra_frame_period(IntraFramePeriod::from_num_frames(fps * 2));
        let encoder = Encoder::with_api_config(OpenH264API::from_source(), config).map_err(|e| format!("cannot start the H.264 encoder: {e}"))?;
        Ok(Video { encoder, mp4: None, file: Some(file), width, height, fps, frames: 0 })
    }

    /// A picture as RGBA bytes at the video's size: scaled down by the whole factor that makes its
    /// width fit (each output pixel the average of a block), then cut or padded to the exact size
    /// (should the window have changed size while recording).
    pub(super) fn picture(&self, image: &egui::ColorImage) -> Vec<u8> {
        let [iw, ih] = image.size;
        let (w, h) = (self.width as usize, self.height as usize);
        let factor = (iw / w.max(1)).max(1);
        let mut out = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let (mut sum, mut n) = ([0u32; 4], 0u32);
                for dy in 0..factor {
                    let sy = y * factor + dy;
                    if sy >= ih {
                        break;
                    }
                    for dx in 0..factor {
                        let sx = x * factor + dx;
                        if sx >= iw {
                            break;
                        }
                        let c = image.pixels[sy * iw + sx].to_array();
                        for (s, v) in sum.iter_mut().zip(c) {
                            *s += v as u32;
                        }
                        n += 1;
                    }
                }
                if n > 0 {
                    let at = (y * w + x) * 4;
                    for (k, s) in sum.iter().enumerate() {
                        out[at + k] = (s / n) as u8;
                    }
                    out[at + 3] = 255; // a video has no transparency
                }
            }
        }
        out
    }

    /// Encode one video frame from `rgba` (at the video's size) and write it.
    pub(super) fn push(&mut self, rgba: &[u8]) -> Result<(), String> {
        let yuv = YUVBuffer::from_rgba8_source(RgbaSliceU8::new(rgba, (self.width as usize, self.height as usize)));
        let stream = self.encoder.encode(&yuv).map_err(|e| format!("encoding failed: {e}"))?;
        let keyframe = matches!(stream.frame_type(), FrameType::IDR | FrameType::I);
        let (mut sps, mut pps, mut sample) = (None, None, Vec::new());
        for l in 0..stream.num_layers() {
            let Some(layer) = stream.layer(l) else { continue };
            for n in 0..layer.nal_count() {
                let Some(nal) = layer.nal_unit(n).map(strip_start_code) else { continue };
                match nal.first().map(|b| b & 0x1f) {
                    Some(7) => sps = Some(nal.to_vec()),
                    Some(8) => pps = Some(nal.to_vec()),
                    Some(_) => {
                        sample.extend_from_slice(&(nal.len() as u32).to_be_bytes());
                        sample.extend_from_slice(nal);
                    }
                    None => {}
                }
            }
        }
        if self.mp4.is_none() {
            // the first frame: start the file with a track built from its parameter sets
            let (Some(sps), Some(pps)) = (sps, pps) else { return Err("the encoder gave no parameter sets".into()) };
            self.mp4 = Some(self.start_file(sps, pps)?);
        }
        if sample.is_empty() {
            return Err("the encoder gave an empty frame".into());
        }
        let duration = TIMESCALE / self.fps;
        let mp4_sample = mp4::Mp4Sample { start_time: self.frames * duration as u64, duration, rendering_offset: 0, is_sync: keyframe, bytes: bytes::Bytes::from(sample) };
        self.mp4.as_mut().expect("started above").write_sample(1, &mp4_sample).map_err(|e| format!("writing the video failed: {e}"))?;
        self.frames += 1;
        Ok(())
    }

    fn start_file(&mut self, sps: Vec<u8>, pps: Vec<u8>) -> Result<mp4::Mp4Writer<BufWriter<std::fs::File>>, String> {
        let brand = |s: &str| s.parse::<mp4::FourCC>().expect("a four-letter brand");
        let config = mp4::Mp4Config { major_brand: brand("isom"), minor_version: 512, compatible_brands: vec![brand("isom"), brand("iso2"), brand("avc1"), brand("mp41")], timescale: 1000 };
        let file = self.file.take().ok_or("the file is already in use")?;
        let mut writer = mp4::Mp4Writer::write_start(file, &config).map_err(|e| format!("writing the video failed: {e}"))?;
        let track = mp4::TrackConfig {
            track_type: mp4::TrackType::Video,
            timescale: TIMESCALE,
            language: "und".into(),
            media_conf: mp4::MediaConfig::AvcConfig(mp4::AvcConfig { width: self.width as u16, height: self.height as u16, seq_param_set: sps, pic_param_set: pps }),
        };
        writer.add_track(&track).map_err(|e| format!("writing the video failed: {e}"))?;
        Ok(writer)
    }

    /// Close the file; returns (frames, width, height).
    pub(super) fn finish(mut self) -> Result<(u64, u32, u32), String> {
        let mut writer = self.mp4.take().ok_or("no frame was encoded")?;
        writer.write_end().map_err(|e| format!("finishing the video failed: {e}"))?;
        writer.into_writer().flush().map_err(|e| format!("finishing the video failed: {e}"))?;
        Ok((self.frames, self.width, self.height))
    }
}

/// A NAL unit without its Annex B start code (`00 00 01` or `00 00 00 01`).
fn strip_start_code(nal: &[u8]) -> &[u8] {
    if nal.starts_with(&[0, 0, 0, 1]) {
        &nal[4..]
    } else if nal.starts_with(&[0, 0, 1]) {
        &nal[3..]
    } else {
        nal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_retina_window_is_halved_and_sizes_are_even() {
        assert_eq!(video_size([2800, 1698], 1920), (1400, 848));
        assert_eq!(video_size([1400, 849], 1920), (1400, 848));
        assert_eq!(video_size([1080, 1920], 1920), (1080, 1920));
        assert_eq!(video_size([4000, 2000], 1920), (1332, 666));
    }

    #[test]
    fn start_codes_are_removed() {
        assert_eq!(strip_start_code(&[0, 0, 0, 1, 0x67, 1]), &[0x67, 1]);
        assert_eq!(strip_start_code(&[0, 0, 1, 0x68]), &[0x68]);
        assert_eq!(strip_start_code(&[0x65, 9]), &[0x65, 9]);
    }
}
