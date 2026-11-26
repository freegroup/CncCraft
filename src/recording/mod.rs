//! Recording the main window as a video: every frame the app draws — the drawing, the virtual
//! mouse, its status panel and the captions — becomes a frame of an H.264 video in an MP4 file, the
//! format TikTok and every player take. Nothing outside the program is called: the frames come from
//! egui's own screenshot of the window, are encoded with the OpenH264 encoder built into the app and
//! written by a pure-Rust MP4 writer.
//!
//! The video has a fixed frame rate. The app draws at its own pace, so each captured picture stands
//! for as many video frames as passed since the last one: a picture drawn late is repeated, extra
//! pictures are skipped — the video plays in real time. Encoding runs on a thread of its own so the
//! app never stutters while recording; [`Recorder::stop`] waits for it to finish the file.
//!
//! This module knows nothing about the app: the host asks [`Recorder::wants_frame`] whether to take a
//! screenshot, hands each picture to [`Recorder::frame`], and stops it.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, SyncSender, TrySendError};
use std::sync::Arc;
use std::thread::JoinHandle;

mod encode;

/// The default frame rate (frames per second).
pub const DEFAULT_FPS: u32 = 30;
/// The default largest side of the video in pixels: bigger windows (a Retina screen draws two pixels
/// per point) are scaled down by a whole factor until they fit.
pub const DEFAULT_MAX_SIDE: u32 = 1920;

/// What a finished recording holds.
#[derive(Clone, Debug, PartialEq)]
pub struct Summary {
    pub path: PathBuf,
    pub frames: u64,
    pub seconds: f64,
    pub width: u32,
    pub height: u32,
}

/// One picture for the encoder and how many video frames it stands for.
struct Frame {
    image: Arc<egui::ColorImage>,
    repeat: u32,
}

/// A recording in progress.
pub struct Recorder {
    path: PathBuf,
    fps: u32,
    tx: Option<SyncSender<Frame>>,
    worker: Option<JoinHandle<Result<Summary, String>>>,
    /// When the first picture was taken (egui time, seconds); the clock of the video.
    started: Option<f64>,
    /// The next video frame still to be filled.
    next_frame: u64,
}

impl Recorder {
    /// Start recording into `path` (an `.mp4` file; its folder is created if needed) at `fps` frames
    /// per second, the video's longest side at most `max_side` pixels.
    pub fn start(path: &Path, fps: u32, max_side: u32) -> Result<Recorder, String> {
        if path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase).as_deref() != Some("mp4") {
            return Err(format!("the file must end in .mp4: {}", path.display()));
        }
        if !(1..=60).contains(&fps) {
            return Err(format!("fps must be between 1 and 60, not {fps}"));
        }
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
        }
        // open the file now, so a bad path is reported at the start, not at the end
        let file = std::fs::File::create(path).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        // a few pictures may wait for the encoder; if it falls further behind, pictures are skipped
        // (the next one stands for the skipped frames) rather than piling up in memory
        let (tx, rx) = std::sync::mpsc::sync_channel::<Frame>(6);
        let (owned, max_side) = (path.to_path_buf(), max_side.max(64));
        let worker = std::thread::Builder::new()
            .name("cnccraft-recorder".into())
            .spawn(move || encode_all(rx, file, owned, fps, max_side))
            .map_err(|e| format!("cannot start the encoder: {e}"))?;
        Ok(Recorder { path: path.to_path_buf(), fps, tx: Some(tx), worker: Some(worker), started: None, next_frame: 0 })
    }

    /// The file being written.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Whether a picture taken at `time` (egui time, seconds) is due: a video frame has come round
    /// since the last one. Taking a screenshot only then keeps the work to the frame rate.
    pub fn wants_frame(&self, time: f64) -> bool {
        match self.started {
            None => true,
            Some(start) => self.frame_at(start, time) >= self.next_frame,
        }
    }

    /// The video frame a picture at `time` falls in.
    fn frame_at(&self, start: f64, time: f64) -> u64 {
        ((time - start).max(0.0) * self.fps as f64).floor() as u64
    }

    /// Hand over a picture of the window taken at `time`. It fills every video frame from the next
    /// one up to `time`; a picture that comes before the next frame is due is not needed.
    pub fn frame(&mut self, image: Arc<egui::ColorImage>, time: f64) {
        let start = *self.started.get_or_insert(time);
        let due = self.frame_at(start, time) + 1; // frames 0..=frame_at are covered after this one
        if due <= self.next_frame {
            return;
        }
        let repeat = (due - self.next_frame) as u32;
        let Some(tx) = &self.tx else { return };
        match tx.try_send(Frame { image, repeat }) {
            Ok(()) => self.next_frame = due,
            // the encoder is busy: leave the frames open, the next picture fills them
            Err(TrySendError::Full(_)) => {}
            Err(TrySendError::Disconnected(_)) => self.tx = None,
        }
    }

    /// Stop: let the encoder finish what it has, close the file and say what it holds.
    pub fn stop(mut self) -> Result<Summary, String> {
        self.tx = None; // the encoder sees the end of the pictures
        match self.worker.take() {
            Some(worker) => worker.join().map_err(|_| "the encoder stopped unexpectedly".to_string())?,
            None => Err("the recording was not running".into()),
        }
    }
}

impl Drop for Recorder {
    /// A recording dropped without `stop` (the program closes) still finishes its file, so the video
    /// up to then can be played.
    fn drop(&mut self) {
        self.tx = None;
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

/// The encoder thread: every picture in, the finished file out.
fn encode_all(rx: Receiver<Frame>, file: std::fs::File, path: PathBuf, fps: u32, max_side: u32) -> Result<Summary, String> {
    let mut video: Option<encode::Video> = None;
    let mut writer = Some(std::io::BufWriter::new(file));
    for frame in rx {
        if video.is_none() {
            // the size is fixed by the first picture
            let (w, h) = encode::video_size(frame.image.size, max_side);
            video = Some(encode::Video::new(writer.take().expect("the file is handed over once"), w, h, fps)?);
        }
        let v = video.as_mut().expect("set just above");
        let rgba = v.picture(&frame.image);
        for _ in 0..frame.repeat {
            v.push(&rgba)?;
        }
    }
    match video {
        Some(v) => {
            let (frames, width, height) = v.finish()?;
            Ok(Summary { path, frames, seconds: frames as f64 / fps as f64, width, height })
        }
        None => {
            // nothing was drawn while recording: no video to keep
            drop(writer);
            let _ = std::fs::remove_file(&path);
            Err("no picture was recorded".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A plain picture of the given size and colour.
    fn picture(w: usize, h: usize, shade: u8) -> Arc<egui::ColorImage> {
        Arc::new(egui::ColorImage::new([w, h], egui::Color32::from_rgb(shade, 255 - shade, 128)))
    }

    #[test]
    fn a_recording_makes_a_playable_mp4_of_the_right_length() {
        let path = std::env::temp_dir().join(format!("cnccraft-recording-test-{}.mp4", std::process::id()));
        let mut rec = Recorder::start(&path, 10, 1920).unwrap();
        // one second of pictures drawn at about 20 per second: 10 video frames
        for i in 0..21 {
            rec.frame(picture(320, 240, (i * 12) as u8), i as f64 * 0.05);
            // give the encoder time, as the app's frames would
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        let summary = rec.stop().unwrap();
        assert_eq!((summary.width, summary.height), (320, 240));
        assert_eq!(summary.frames, 11, "frames 0..=10 of one second at 10 fps");
        // the file is an MP4 with one H.264 video track of that many samples
        let bytes = std::fs::read(&path).unwrap();
        let size = bytes.len() as u64;
        let mp4 = mp4::Mp4Reader::read_header(std::io::Cursor::new(bytes), size).unwrap();
        let track = mp4.tracks().values().next().unwrap();
        assert_eq!(track.media_type().unwrap(), mp4::MediaType::H264);
        assert_eq!(track.sample_count(), 11);
        assert_eq!((track.width(), track.height()), (320, 240));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_late_picture_fills_the_frames_it_missed() {
        let path = std::env::temp_dir().join(format!("cnccraft-recording-gap-{}.mp4", std::process::id()));
        let mut rec = Recorder::start(&path, 30, 1920).unwrap();
        rec.frame(picture(64, 64, 0), 0.0);
        assert!(!rec.wants_frame(0.01), "the next frame is not due yet");
        assert!(rec.wants_frame(0.04));
        std::thread::sleep(std::time::Duration::from_millis(20));
        rec.frame(picture(64, 64, 100), 0.5); // half a second later: frames 1..=15
        let summary = rec.stop().unwrap();
        assert_eq!(summary.frames, 16);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn only_mp4_files_are_written() {
        assert!(Recorder::start(Path::new("/tmp/x.mov"), 30, 1920).is_err());
        assert!(Recorder::start(Path::new("/tmp/x.mp4"), 0, 1920).is_err());
    }
}
