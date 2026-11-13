//! Importing drawings from files of other programs. Every format has its own file in this folder
//! with a `read` function that turns the file's bytes into shapes; `FORMATS` lists them, so a new
//! format is one new file and one line there. Plain logic without a window, so it can be tested.

mod dxf;

use shape_edit::Shape;
use std::path::Path;

/// What reading a file brought in.
#[derive(Debug)]
pub struct Import {
    /// The shapes, in millimetres, as they lie in the file.
    pub shapes: Vec<Shape>,
    /// The version of the program that wrote the file ("R12", "2018" …); empty if the format has none.
    pub version: String,
    /// The drawing unit the file named (the shapes are converted to millimetres already).
    pub unit: &'static str,
    /// Kinds of things that were left out, with how many of each.
    pub skipped: Vec<(String, usize)>,
}

/// A file format that can be imported.
pub struct Format {
    /// Short name, kept on every shape that came from such a file.
    pub name: &'static str,
    /// What the file dialog calls it.
    pub description: &'static str,
    pub extensions: &'static [&'static str],
    pub read: fn(&[u8]) -> Result<Import, String>,
}

pub const FORMATS: &[Format] = &[Format { name: "DXF", description: "DXF-Zeichnung (R12, R14 und neuer)", extensions: &["dxf"], read: dxf::read }];

/// The format a file name belongs to, by its extension.
pub fn format_of(path: &Path) -> Option<&'static Format> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    FORMATS.iter().find(|f| f.extensions.contains(&ext.as_str()))
}

/// Read a file with the reader of its format.
pub fn read_file(path: &Path) -> Result<(&'static Format, Import), String> {
    let format = format_of(path).ok_or_else(|| t!("Dieses Dateiformat kann nicht importiert werden.").to_string())?;
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    Ok((format, (format.read)(&bytes)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_format_comes_from_the_extension() {
        assert_eq!(format_of(Path::new("/tmp/Teil.DXF")).map(|f| f.name), Some("DXF"));
        assert!(format_of(Path::new("/tmp/teil.svg")).is_none());
        assert!(format_of(Path::new("/tmp/ohne")).is_none());
    }

    #[test]
    fn a_missing_file_is_an_error() {
        assert!(read_file(Path::new("/nicht/da/teil.dxf")).is_err());
    }
}
