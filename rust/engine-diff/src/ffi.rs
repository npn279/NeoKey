//! All calls to the independent, original C++ implementation live in this module.
use neokey_engine::{ConvertOptions, HookState, Settings};
use std::ffi::{CStr, CString};
extern "C" {
    fn ref_key_character(d: u32) -> u16;
    fn ref_settings(v: *const i64);
    fn ref_reset();
    fn ref_event(e: u8, s: u8, d: u16, c: u8, o: bool) -> u8;
    fn ref_english(s: u8, d: u16, c: bool, o: bool);
    fn ref_session();
    fn ref_temp_spell();
    fn ref_temp_engine(off: bool);
    fn ref_character(d: u32) -> u32;
    fn ref_hook(h: *mut u8, c: *mut u32);
    fn ref_macro(key: bool, n: *mut usize) -> *const u32;
    fn ref_add_macro(k: *const i8, v: *const i8) -> bool;
    fn ref_delete_macro(k: *const i8) -> bool;
    fn ref_init_macro(b: *const u8, n: i32);
    fn ref_save_macro(n: *mut usize) -> *const u8;
    fn ref_init_smart(b: *const u8, n: i32);
    fn ref_get_smart(k: *const i8, c: i32) -> i32;
    fn ref_set_smart(k: *const i8, v: i32);
    fn ref_save_smart(n: *mut usize) -> *const u8;
    fn ref_convert(t: *const i8, f: u8, to: u8, flags: u32) -> *const i8;
}
pub fn settings(v: &Settings) {
    unsafe { ref_settings(v as *const Settings as *const i64) }
}
pub fn reset() {
    unsafe { ref_reset() }
}
pub fn event(e: u8, s: u8, d: u16, c: u8, o: bool) -> u8 {
    unsafe { ref_event(e, s, d, c, o) }
}
pub fn english(s: u8, d: u16, c: bool, o: bool) {
    unsafe { ref_english(s, d, c, o) }
}
pub fn session() {
    unsafe { ref_session() }
}
pub fn temp_spell() {
    unsafe { ref_temp_spell() }
}
pub fn temp_engine(off: bool) {
    unsafe { ref_temp_engine(off) }
}
pub fn character(d: u32) -> u32 {
    unsafe { ref_character(d) }
}
pub fn hook() -> HookState {
    unsafe {
        let mut h = [0; 4];
        let mut chars = [0; 32];
        ref_hook(h.as_mut_ptr(), chars.as_mut_ptr());
        let mut vectors = Vec::new();
        for key in [true, false] {
            let mut n = 0;
            let p = ref_macro(key, &mut n);
            vectors.push(if n == 0 {
                vec![]
            } else {
                std::slice::from_raw_parts(p, n).to_vec()
            });
        }
        HookState {
            code: h[0],
            backspace_count: h[1],
            new_char_count: h[2],
            ext_code: h[3],
            char_data: chars,
            macro_key: vectors.remove(0),
            macro_data: vectors.remove(0),
        }
    }
}
pub fn add_macro(k: &str, v: &str) {
    unsafe {
        assert!(ref_add_macro(
            CString::new(k).unwrap().as_ptr(),
            CString::new(v).unwrap().as_ptr()
        ))
    }
}
pub fn delete_macro(k: &str) -> bool {
    unsafe { ref_delete_macro(CString::new(k).unwrap().as_ptr()) }
}
pub fn init_macro(b: &[u8]) {
    unsafe { ref_init_macro(b.as_ptr(), b.len() as i32) }
}
pub fn save_macro() -> Vec<u8> {
    unsafe {
        let mut n = 0;
        let p = ref_save_macro(&mut n);
        std::slice::from_raw_parts(p, n).to_vec()
    }
}
pub fn init_smart(b: &[u8]) {
    unsafe { ref_init_smart(b.as_ptr(), b.len() as i32) }
}
pub fn get_smart(k: &str, c: i32) -> i32 {
    unsafe { ref_get_smart(CString::new(k).unwrap().as_ptr(), c) }
}
pub fn set_smart(k: &str, v: i32) {
    unsafe { ref_set_smart(CString::new(k).unwrap().as_ptr(), v) }
}
pub fn save_smart() -> Vec<u8> {
    unsafe {
        let mut n = 0;
        let p = ref_save_smart(&mut n);
        std::slice::from_raw_parts(p, n).to_vec()
    }
}
pub fn convert(
    t: &str,
    o: &ConvertOptions,
) -> Result<String, neokey_engine::convert::ConversionError> {
    unsafe {
        let flags = o.to_all_caps as u32
            | (o.to_all_non_caps as u32) << 1
            | (o.to_caps_first_letter as u32) << 2
            | (o.to_caps_each_word as u32) << 3
            | (o.remove_mark as u32) << 4;
        let p = ref_convert(
            CString::new(t).unwrap().as_ptr(),
            o.from_code,
            o.to_code,
            flags,
        );
        if p.is_null() {
            Err(neokey_engine::convert::ConversionError)
        } else {
            Ok(CStr::from_ptr(p).to_str().unwrap().into())
        }
    }
}

pub fn key_character(d: u32) -> u16 {
    unsafe { ref_key_character(d) }
}
