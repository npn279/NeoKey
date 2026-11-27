use std::ffi::c_void;
use std::ptr::NonNull;

use objc2_core_foundation::{CFArray, CFRetained, CFString, CFType};

#[link(name = "Carbon", kind = "framework")]
extern "C" {
    fn TISCopyCurrentKeyboardInputSource() -> *mut CFType;
    fn TISGetInputSourceProperty(source: &CFType, key: &CFString) -> *const c_void;
    static kTISPropertyInputSourceLanguages: &'static CFString;
}

pub fn is_non_english() -> bool {
    // TIS returns an owned source and a borrowed array whose lifetime is tied to it.
    // The languages property is a CFArray of BCP 47 CFString language tags.
    unsafe {
        let Some(source) = NonNull::new(TISCopyCurrentKeyboardInputSource()) else {
            return false;
        };
        let source = CFRetained::from_raw(source);
        let languages = TISGetInputSourceProperty(&source, kTISPropertyInputSourceLanguages)
            .cast::<CFArray<CFString>>()
            .as_ref();
        languages
            .and_then(|languages| languages.get(0))
            .is_some_and(|language| !language.to_string().starts_with("en"))
    }
}
