//! A minimal ZIP writer: files are stored without compression (G-code files are small), which
//! every unzip program reads. Used to hand several machine files over as one download.

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xedb8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

/// MS-DOS time and date of a moment given as seconds since 1970 (UTC).
fn dos_time(seconds: u64) -> (u16, u16) {
    let days = (seconds / 86_400) as i64;
    let rest = seconds % 86_400;
    // civil date from days since 1970-01-01 (Howard Hinnant's algorithm)
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    let year = year.clamp(1980, 2107);
    let date = (((year - 1980) as u16) << 9) | ((month as u16) << 5) | day as u16;
    let time = (((rest / 3600) as u16) << 11) | ((((rest % 3600) / 60) as u16) << 5) | ((rest % 60 / 2) as u16);
    (time, date)
}

/// The bytes of a ZIP archive holding the given files (name, content). `seconds` is the time stamp
/// of every file, as seconds since 1970 (UTC).
pub fn stored(files: &[(String, Vec<u8>)], seconds: u64) -> Vec<u8> {
    let (time, date) = dos_time(seconds);
    let mut out: Vec<u8> = Vec::new();
    let mut central: Vec<u8> = Vec::new();
    for (name, data) in files {
        let offset = out.len() as u32;
        let crc = crc32(data);
        let size = data.len() as u32;
        let name = name.as_bytes();
        // local file header
        out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes()); // version needed
        out.extend_from_slice(&0x0800u16.to_le_bytes()); // names are UTF-8
        out.extend_from_slice(&0u16.to_le_bytes()); // stored
        out.extend_from_slice(&time.to_le_bytes());
        out.extend_from_slice(&date.to_le_bytes());
        out.extend_from_slice(&crc.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(name);
        out.extend_from_slice(data);
        // central directory entry
        central.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
        central.extend_from_slice(&20u16.to_le_bytes()); // made by
        central.extend_from_slice(&20u16.to_le_bytes()); // version needed
        central.extend_from_slice(&0x0800u16.to_le_bytes());
        central.extend_from_slice(&0u16.to_le_bytes());
        central.extend_from_slice(&time.to_le_bytes());
        central.extend_from_slice(&date.to_le_bytes());
        central.extend_from_slice(&crc.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&size.to_le_bytes());
        central.extend_from_slice(&(name.len() as u16).to_le_bytes());
        central.extend_from_slice(&[0; 12]); // extra, comment, disk, internal and external attributes
        central.extend_from_slice(&offset.to_le_bytes());
        central.extend_from_slice(name);
    }
    let start = out.len() as u32;
    out.extend_from_slice(&central);
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&[0; 4]); // disk numbers
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(files.len() as u16).to_le_bytes());
    out.extend_from_slice(&(central.len() as u32).to_le_bytes());
    out.extend_from_slice(&start.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u16_at(b: &[u8], i: usize) -> usize {
        u16::from_le_bytes([b[i], b[i + 1]]) as usize
    }
    fn u32_at(b: &[u8], i: usize) -> usize {
        u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]) as usize
    }

    #[test]
    fn the_checksum_is_the_standard_one() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
        assert_eq!(crc32(b""), 0);
    }

    #[test]
    fn the_date_is_right() {
        // 2026-11-12 17:15:13 UTC
        let (time, date) = dos_time(1_794_503_713);
        assert_eq!((date >> 9) + 1980, 2026);
        assert_eq!((date >> 5) & 15, 11);
        assert_eq!(date & 31, 12);
        assert_eq!(time >> 11, 17);
        assert_eq!((time >> 5) & 63, 15);
    }

    #[test]
    fn the_archive_can_be_read_back() {
        let files = vec![("a_01_x.nc".to_string(), b"G21\nM30\n".to_vec()), ("a_02_y.nc".to_string(), b"G20\n".to_vec())];
        let zip = stored(&files, 1_794_503_713);
        // end of central directory: entry count, directory size and start
        let end = zip.len() - 22;
        assert_eq!(u32_at(&zip, end), 0x0605_4b50);
        assert_eq!(u16_at(&zip, end + 10), 2);
        let (dir_len, dir_start) = (u32_at(&zip, end + 12), u32_at(&zip, end + 16));
        assert_eq!(dir_start + dir_len, end);
        // walk the central directory and read each file through its local header
        let mut at = dir_start;
        for (name, data) in &files {
            assert_eq!(u32_at(&zip, at), 0x0201_4b50);
            let (crc, size, name_len, local) = (u32_at(&zip, at + 16), u32_at(&zip, at + 24), u16_at(&zip, at + 28), u32_at(&zip, at + 42));
            assert_eq!(&zip[at + 46..at + 46 + name_len], name.as_bytes());
            assert_eq!(size, data.len());
            assert_eq!(crc as u32, crc32(data));
            assert_eq!(u32_at(&zip, local), 0x0403_4b50);
            let body = local + 30 + u16_at(&zip, local + 26) + u16_at(&zip, local + 28);
            assert_eq!(&zip[body..body + size], &data[..]);
            at += 46 + name_len;
        }
    }
}
