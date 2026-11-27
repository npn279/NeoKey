//! C bridge. Legacy entry points use one engine per calling thread.
//! The HookState pointer is stable until thread exit; vector pointers last until the next mutating call.
//! Raw buffers must be valid for their stated length. Only this module uses unsafe Rust.
#![allow(non_snake_case)]
use crate::{ConvertOptions, Engine, Event, EventState, Settings, SmartSwitch};
use std::{
    cell::RefCell,
    ffi::{c_char, CStr},
    ptr,
};

#[repr(C)]
pub struct CHookState {
    pub code: u8,
    pub backspaceCount: u8,
    pub newCharCount: u8,
    pub extCode: u8,
    pub charData: [u32; 32],
    pub macroKey: *const u32,
    pub macroKeyCount: usize,
    pub macroData: *const u32,
    pub macroDataCount: usize,
}
impl Default for CHookState {
    fn default() -> Self {
        Self {
            code: 0,
            backspaceCount: 0,
            newCharCount: 0,
            extCode: 0,
            charData: [0; 32],
            macroKey: ptr::null(),
            macroKeyCount: 0,
            macroData: ptr::null(),
            macroDataCount: 0,
        }
    }
}
struct Context {
    engine: Engine,
    hook: Box<CHookState>,
    smart: SmartSwitch,
    buffer: Vec<u8>,
    convert: ConvertOptions,
}
impl Default for Context {
    fn default() -> Self {
        Self {
            engine: Engine::new(),
            hook: Box::default(),
            smart: SmartSwitch::default(),
            buffer: Vec::new(),
            convert: ConvertOptions::default(),
        }
    }
}
impl Context {
    fn sync(&mut self) -> *const CHookState {
        let r = self.engine.hook_state();
        let h = &mut self.hook;
        h.code = r.code;
        h.backspaceCount = r.backspace_count;
        h.newCharCount = r.new_char_count;
        h.extCode = r.ext_code;
        h.charData = r.char_data;
        h.macroKey = r.macro_key.as_ptr();
        h.macroKeyCount = r.macro_key.len();
        h.macroData = r.macro_data.as_ptr();
        h.macroDataCount = r.macro_data.len();
        &**h
    }
}
thread_local! {static CONTEXT:RefCell<Context> = RefCell::new(Context::default());}
fn state(value: u8) -> Option<EventState> {
    Some(match value {
        0 => EventState::KeyDown,
        1 => EventState::KeyUp,
        2 => EventState::MouseDown,
        3 => EventState::MouseUp,
        _ => return None,
    })
}
fn valid_settings(s: &Settings) -> bool {
    (0..=3).contains(&s.vInputType) && (0..=4).contains(&s.vCodeTable)
}
// SAFETY: callers of exported unsafe functions provide the documented readable buffers.
unsafe fn bytes<'a>(p: *const u8, n: usize) -> Option<&'a [u8]> {
    if n == 0 {
        Some(&[])
    } else if p.is_null() || n > isize::MAX as usize {
        None
    } else {
        Some(unsafe { std::slice::from_raw_parts(p, n) })
    }
}
unsafe fn text<'a>(p: *const c_char) -> Option<&'a str> {
    if p.is_null() {
        None
    } else {
        unsafe { CStr::from_ptr(p) }.to_str().ok()
    }
}

#[no_mangle]
pub extern "C" fn vKeyInit() -> *const CHookState {
    CONTEXT.with(|c| {
        let mut c = c.borrow_mut();
        c.engine.initialize();
        c.sync()
    })
}
#[no_mangle]
pub extern "C" fn vKeyGetHookState() -> *const CHookState {
    CONTEXT.with(|c| c.borrow_mut().sync())
}
#[no_mangle]
pub extern "C" fn vKeyGetSettings() -> Settings {
    CONTEXT.with(|c| c.borrow().engine.settings().clone())
}
/// # Safety
/// `settings` points to one initialized Settings value.
#[no_mangle]
pub unsafe extern "C" fn vKeySetSettings(settings: *const Settings) -> bool {
    let Some(s) = (unsafe { settings.as_ref() }) else {
        return false;
    };
    if !valid_settings(s) {
        return false;
    }
    CONTEXT.with(|c| c.borrow_mut().engine.set_settings(s.clone()));
    true
}
#[no_mangle]
pub extern "C" fn vKeyDefaultSettings() -> Settings {
    Settings::default()
}
#[no_mangle]
pub extern "C" fn vKeyHandleEvent(
    event: u8,
    event_state: u8,
    data: u16,
    caps_status: u8,
    other_control_key: bool,
) {
    let Some(s) = state(event_state) else {
        return;
    };
    let e = match event {
        0 => Event::Keyboard,
        1 => Event::Mouse,
        _ => return,
    };
    CONTEXT.with(|c| {
        let mut c = c.borrow_mut();
        c.engine
            .handle_event(e, s, data, caps_status, other_control_key);
        c.sync();
    });
}
#[no_mangle]
pub extern "C" fn startNewSession() {
    CONTEXT.with(|c| {
        let mut c = c.borrow_mut();
        c.engine.start_new_session();
        c.sync();
    });
}
#[no_mangle]
pub extern "C" fn getCharacterCode(data: u32) -> u32 {
    CONTEXT.with(|c| c.borrow_mut().engine.get_character_code(data))
}
#[no_mangle]
pub extern "C" fn vEnglishMode(event_state: u8, data: u16, is_caps: bool, other_control_key: bool) {
    let Some(s) = state(event_state) else {
        return;
    };
    CONTEXT.with(|c| {
        let mut c = c.borrow_mut();
        c.engine.english_mode(s, data, is_caps, other_control_key);
        c.sync();
    });
}
#[no_mangle]
pub extern "C" fn vTempOffSpellChecking() {
    CONTEXT.with(|c| c.borrow_mut().engine.temp_off_spell_checking());
}
#[no_mangle]
pub extern "C" fn vSetCheckSpelling() {
    CONTEXT.with(|c| c.borrow_mut().engine.set_check_spelling());
}
#[no_mangle]
pub extern "C" fn vTempOffEngine(off: bool) {
    CONTEXT.with(|c| c.borrow_mut().engine.temp_off_engine(off));
}
/// # Safety
/// `data` is readable for `size` bytes (or null when size is zero).
#[no_mangle]
pub unsafe extern "C" fn initMacroMap(data: *const u8, size: usize) -> bool {
    let Some(data) = (unsafe { bytes(data, size) }) else {
        return false;
    };
    CONTEXT.with(|c| c.borrow_mut().engine.init_macros(data).is_ok())
}
/// # Safety
/// `key` and `content` are NUL-terminated UTF-8 strings.
#[no_mangle]
pub unsafe extern "C" fn addMacro(key: *const c_char, content: *const c_char) -> bool {
    let (Some(key), Some(content)) = (unsafe { text(key) }, unsafe { text(content) }) else {
        return false;
    };
    CONTEXT.with(|c| c.borrow_mut().engine.add_macro(key, content))
}
/// # Safety
/// `key` is a NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn deleteMacro(key: *const c_char) -> bool {
    let Some(key) = (unsafe { text(key) }) else {
        return false;
    };
    CONTEXT.with(|c| c.borrow_mut().engine.delete_macro(key))
}
/// # Safety
/// `key` is a NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn hasMacro(key: *const c_char) -> bool {
    let Some(key) = (unsafe { text(key) }) else {
        return false;
    };
    CONTEXT.with(|c| {
        let c = c.borrow();
        c.engine
            .macros()
            .has(key, c.engine.settings().vCodeTable as usize)
    })
}
/// # Safety
/// `key` points to `count` readable packed codes. Successful lookup updates the hook's macroData.
#[no_mangle]
pub unsafe extern "C" fn findMacro(key: *const u32, count: usize) -> bool {
    if count > isize::MAX as usize / std::mem::size_of::<u32>() || (key.is_null() && count != 0) {
        return false;
    }
    let mut key = if count == 0 {
        vec![]
    } else {
        unsafe { std::slice::from_raw_parts(key, count) }.to_vec()
    };
    CONTEXT.with(|c| {
        let mut c = c.borrow_mut();
        let result = c.engine.find_macro(&mut key);
        let found = result.is_some();
        if let Some(v) = result {
            c.engine.set_macro_output(key, v);
            c.sync();
        }
        found
    })
}
/// # Safety
/// `size` points to writable size_t. Returned bytes last until the next serialization or conversion call.
#[no_mangle]
pub unsafe extern "C" fn getMacroSaveData(size: *mut usize) -> *const u8 {
    if size.is_null() {
        return ptr::null();
    }
    CONTEXT.with(|c| {
        let mut c = c.borrow_mut();
        c.buffer = c.engine.macros().save_data();
        unsafe {
            *size = c.buffer.len();
        }
        c.buffer.as_ptr()
    })
}
#[no_mangle]
pub extern "C" fn onTableCodeChange() {
    CONTEXT.with(|c| {
        let mut c = c.borrow_mut();
        c.engine.on_table_code_change();
    });
}
/// # Safety
/// `data` is readable for `size` bytes (or null when size is zero).
#[no_mangle]
pub unsafe extern "C" fn initSmartSwitchKey(data: *const u8, size: usize) -> bool {
    let Some(data) = (unsafe { bytes(data, size) }) else {
        return false;
    };
    CONTEXT.with(|c| c.borrow_mut().smart.init(data).is_ok())
}
/// # Safety
/// `size` points to writable size_t. Returned bytes last until the next serialization or conversion call.
#[no_mangle]
pub unsafe extern "C" fn getSmartSwitchKeySaveData(size: *mut usize) -> *const u8 {
    if size.is_null() {
        return ptr::null();
    }
    CONTEXT.with(|c| {
        let mut c = c.borrow_mut();
        c.buffer = c.smart.save_data();
        unsafe {
            *size = c.buffer.len();
        }
        c.buffer.as_ptr()
    })
}
/// # Safety
/// `bundle` is a NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn getAppInputMethodStatus(bundle: *const c_char, current: i32) -> i32 {
    let Some(bundle) = (unsafe { text(bundle) }) else {
        return -1;
    };
    CONTEXT.with(|c| c.borrow_mut().smart.get(bundle, current))
}
/// # Safety
/// `bundle` is a NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn setAppInputMethodStatus(bundle: *const c_char, language: i32) {
    if let Some(bundle) = unsafe { text(bundle) } {
        CONTEXT.with(|c| c.borrow_mut().smart.set(bundle, language));
    }
}
/// # Safety
/// `options` points to one initialized ConvertOptions.
#[no_mangle]
pub unsafe extern "C" fn setConvertOptions(options: *const ConvertOptions) -> bool {
    let Some(o) = (unsafe { options.as_ref() }) else {
        return false;
    };
    if o.from_code > 4 || o.to_code > 4 {
        return false;
    }
    CONTEXT.with(|c| c.borrow_mut().convert = o.clone());
    true
}
/// # Safety
/// `source` is a NUL-terminated UTF-8 string. Null means conversion failed.
/// Returned UTF-8 lasts until the next serialization or conversion call.
#[no_mangle]
pub unsafe extern "C" fn convertUtil(source: *const c_char) -> *const c_char {
    let Some(source) = (unsafe { text(source) }) else {
        return ptr::null();
    };
    CONTEXT.with(|c| {
        let mut c = c.borrow_mut();
        let Ok(result) = c.convert.convert(source) else {
            return ptr::null();
        };
        c.buffer = result.into_bytes();
        c.buffer.push(0);
        c.buffer.as_ptr().cast()
    })
}

/// # Safety
/// `path` is a NUL-terminated UTF-8 file path. Returns false on an I/O failure.
#[no_mangle]
pub unsafe extern "C" fn saveToFile(path: *const c_char) -> bool {
    let Some(path) = (unsafe { text(path) }) else {
        return false;
    };
    CONTEXT.with(|c| c.borrow().engine.macros().save_to_file(path).is_ok())
}
/// # Safety
/// `path` is a NUL-terminated UTF-8 file path. Returns false on an I/O failure.
#[no_mangle]
pub unsafe extern "C" fn readFromFile(path: *const c_char, append: bool) -> bool {
    let Some(path) = (unsafe { text(path) }) else {
        return false;
    };
    CONTEXT.with(|c| {
        c.borrow_mut()
            .engine
            .read_macros_from_file(path, append)
            .is_ok()
    })
}
