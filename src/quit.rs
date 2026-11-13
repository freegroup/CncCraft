//! Quitting from outside the window. Closing the window and the red button reach the program as
//! a close request, but "Quit" in the Dock's menu (and ⌘Q) ends a macOS program without asking it.
//! So the program tells the system whether it may quit: with unsaved changes it says no and the
//! program shows its own question instead.

use std::sync::atomic::{AtomicBool, Ordering};

static DIRTY: AtomicBool = AtomicBool::new(false);
static ASKED: AtomicBool = AtomicBool::new(false);
static WAKE: std::sync::OnceLock<egui::Context> = std::sync::OnceLock::new();

/// Keep the system informed whether there are unsaved changes.
pub fn set_dirty(dirty: bool) {
    DIRTY.store(dirty, Ordering::Relaxed);
}

/// Whether the system asked to quit (and was told no) since the last call.
pub fn take_request() -> bool {
    ASKED.swap(false, Ordering::Relaxed)
}

/// Let the system ask the program before it quits.
pub fn install(ctx: &egui::Context) {
    let _ = WAKE.set(ctx.clone());
    #[cfg(target_os = "macos")]
    mac::install();
}

#[cfg(target_os = "macos")]
mod mac {
    use super::{ASKED, DIRTY, WAKE};
    use objc2::ffi::{class_addMethod, objc_class};
    use objc2::rc::Retained;
    use objc2::runtime::{AnyClass, AnyObject, Sel};
    use objc2::sel;
    use objc2_app_kit::NSApplication;
    use objc2_foundation::MainThreadMarker;
    use std::sync::atomic::Ordering;

    const TERMINATE_CANCEL: usize = 0;
    const TERMINATE_NOW: usize = 1;

    extern "C" fn should_terminate(_this: *mut AnyObject, _cmd: Sel, _sender: *mut AnyObject) -> usize {
        if !DIRTY.load(Ordering::Relaxed) {
            return TERMINATE_NOW;
        }
        ASKED.store(true, Ordering::Relaxed);
        if let Some(ctx) = WAKE.get() {
            ctx.request_repaint();
        }
        TERMINATE_CANCEL
    }

    pub fn install() {
        let Some(mtm) = MainThreadMarker::new() else { return };
        let app = NSApplication::sharedApplication(mtm);
        let Some(delegate) = (unsafe { app.delegate() }) else { return };
        // the delegate's class (made by the window library) gets the one method it lacks
        let object: &AnyObject = unsafe { &*Retained::as_ptr(&delegate).cast::<AnyObject>() };
        let class: &AnyClass = object.class();
        unsafe {
            let imp: unsafe extern "C" fn() = std::mem::transmute(should_terminate as extern "C" fn(*mut AnyObject, Sel, *mut AnyObject) -> usize);
            class_addMethod(class as *const AnyClass as *mut objc_class, sel!(applicationShouldTerminate:).as_ptr(), Some(imp), c"Q@:@".as_ptr());
        }
    }
}
