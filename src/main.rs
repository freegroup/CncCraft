//! CncCraft: a CNC design and toolpath program (rebuild of Vectric Spark's core features).

// in a Windows release there is no console window behind the program
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

// every text the person sees goes through `t!`: the German original is the key, the other
// languages are in locales/<code>.json (see lang.rs); a text without a translation shows as is
rust_i18n::i18n!("locales", fallback = "de");

macro_rules! t {
    ($($args:tt)*) => {
        ::rust_i18n::t!($($args)*).into_owned()
    };
}

// a text kept in a table or a struct: translated when it is shown (see `lang::Tr`)
macro_rules! tr {
    ($key:literal) => {
        $crate::lang::Tr::new($key)
    };
}

mod app;
mod appicon;
mod gcode;
mod layers;
mod export;
mod geom;
mod import;
mod lang;
mod props;
mod quit;
mod sheet;
mod sim;
mod theme;
mod toolgroups;
mod toolpath;
mod tools;
mod transform;
mod view3d;
mod viewdeco;
mod zipfile;

use std::path::PathBuf;

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let shot = args.iter().position(|a| a == "--screenshot").and_then(|i| args.get(i + 1)).map(PathBuf::from);
    let selftest = args.iter().any(|a| a == "--selftest");
    // the review runs (screenshot, self-test) start from a clean state every time: what the
    // interface remembers of earlier runs (window places, scroll offsets) would change what they see
    let persistence_path = (shot.is_some() || selftest).then(|| std::env::temp_dir().join(format!("cnccraft-review-{}", std::process::id())).join("app.ron"));
    // `--export-ico file.ico` writes the icon in the sizes Windows wants (for the installer)
    if let Some(path) = args.iter().position(|a| a == "--export-ico").and_then(|i| args.get(i + 1)) {
        let result = (|| -> Result<(), String> {
            use image::codecs::ico::{IcoEncoder, IcoFrame};
            let mut frames = vec![];
            for size in [16u32, 32, 48, 64, 128, 256] {
                let icon = appicon::icon_data(size);
                frames.push(IcoFrame::as_png(&icon.rgba, size, size, image::ExtendedColorType::Rgba8).map_err(|e| e.to_string())?);
            }
            let file = std::fs::File::create(path).map_err(|e| e.to_string())?;
            IcoEncoder::new(file).encode_images(&frames).map_err(|e| e.to_string())
        })();
        match result {
            Ok(()) => println!("Symbol gespeichert: {path}"),
            Err(e) => eprintln!("Symbol nicht gespeichert: {e}"),
        }
        return Ok(());
    }
    // `--export-icon file.png` writes the program's icon as a picture (for a macOS app bundle)
    if let Some(path) = args.iter().position(|a| a == "--export-icon").and_then(|i| args.get(i + 1)) {
        let icon = appicon::icon_data(512);
        match image::save_buffer(path, &icon.rgba, icon.width, icon.height, image::ColorType::Rgba8) {
            Ok(()) => println!("Symbol gespeichert: {path}"),
            Err(e) => eprintln!("Symbol nicht gespeichert: {e}"),
        }
        return Ok(());
    }
    let options = eframe::NativeOptions {
        persistence_path,
        viewport: egui::ViewportBuilder::default().with_title("CncCraft").with_icon(std::sync::Arc::new(appicon::icon_data(256))).with_inner_size([1400.0, 860.0]).with_min_inner_size([1000.0, 640.0]),
        persist_window: false,
        // the 3D preview draws with a depth buffer
        depth_buffer: 24,
        ..Default::default()
    };
    eframe::run_native("CncCraft", options, Box::new(move |cc| Ok(Box::new(app::App::new(cc, shot, selftest)))))
}
