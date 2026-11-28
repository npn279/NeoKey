//! Safe, per-instance port of OpenKey's platform-independent engine.
//! Packed output and macOS virtual key codes retain the original C++ semantics.
#![deny(unsafe_code)]
// The core port keeps upstream parentheses, casts and control flow for auditability.
#![allow(
    clippy::clone_on_copy,
    clippy::collapsible_if,
    clippy::derivable_impls,
    clippy::double_parens,
    clippy::manual_range_contains,
    clippy::mixed_case_hex_literals,
    clippy::needless_return,
    clippy::unnecessary_cast
)]
pub mod convert;
mod core;
pub mod data;
#[allow(unsafe_code)]
pub mod ffi;
pub mod macros;
mod settings;
pub mod smart_switch;
pub mod vietnamese;
pub use convert::ConvertOptions;
pub use settings::Settings;
pub use smart_switch::SmartSwitch;

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Event {
    Keyboard,
    Mouse,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum EventState {
    KeyDown,
    KeyUp,
    MouseDown,
    MouseUp,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HookState {
    pub code: u8,
    pub backspace_count: u8,
    pub new_char_count: u8,
    pub ext_code: u8,
    /// Replacement characters in reverse order, as in vKeyHookState.
    pub char_data: [u32; 32],
    pub macro_key: Vec<u32>,
    pub macro_data: Vec<u32>,
}

#[derive(Clone)]
pub struct Engine {
    core: core::Core,
    hook: HookState,
}
impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}
impl Engine {
    pub fn new() -> Self {
        Self::with_settings(Settings::default())
    }
    pub fn with_settings(settings: Settings) -> Self {
        assert!((0..=3).contains(&settings.vInputType));
        assert!((0..=4).contains(&settings.vCodeTable));
        Self {
            core: core::Core::new(settings),
            hook: HookState::default(),
        }
    }
    pub fn settings(&self) -> &Settings {
        &self.core.settings
    }
    /// Updating settings also rebuilds macro contents and the spelling toggle baseline.
    pub fn set_settings(&mut self, settings: Settings) {
        assert!((0..=3).contains(&settings.vInputType));
        assert!((0..=4).contains(&settings.vCodeTable));
        self.core.settings = settings;
        self.core.vSetCheckSpelling();
        self.core
            .macros
            .on_table_code_change(self.core.settings.vCodeTable as usize);
    }
    pub fn handle_event(
        &mut self,
        event: Event,
        state: EventState,
        data: u16,
        cap_status: u8,
        other_control_key: bool,
    ) -> &HookState {
        self.core.vKeyHandleEvent(
            event as i64,
            state as i64,
            data as i64,
            cap_status as i64,
            other_control_key as i64,
        );
        self.sync_hook()
    }
    /// Reset the same session/history fields as vKeyInit; retain settings and macros.
    pub fn initialize(&mut self) {
        self.core.initialize();
        self.sync_hook();
    }
    pub fn start_new_session(&mut self) {
        self.core.startNewSession();
        self.sync_hook();
    }
    pub fn english_mode(
        &mut self,
        state: EventState,
        data: u16,
        is_caps: bool,
        other_control_key: bool,
    ) -> &HookState {
        self.core.vEnglishMode(
            state as i64,
            data as i64,
            is_caps as i64,
            other_control_key as i64,
        );
        self.sync_hook()
    }
    pub fn temp_off_spell_checking(&mut self) {
        self.core.vTempOffSpellChecking();
    }
    pub fn set_check_spelling(&mut self) {
        self.core.vSetCheckSpelling();
    }
    pub fn temp_off_engine(&mut self, off: bool) {
        self.core.vTempOffEngine(off as i64);
    }
    pub fn get_character_code(&mut self, data: u32) -> u32 {
        self.core.getCharacterCode(data as i64) as u32
    }
    pub fn hook_state(&self) -> &HookState {
        &self.hook
    }
    pub fn add_macro(&mut self, key: &str, content: &str) -> bool {
        self.core
            .macros
            .add(key, content, self.core.settings.vCodeTable as usize)
    }
    pub fn init_macros(&mut self, data: &[u8]) -> Result<(), macros::InvalidData> {
        self.core
            .macros
            .init(data, self.core.settings.vCodeTable as usize)
    }
    pub fn read_macros_from_file(
        &mut self,
        path: impl AsRef<std::path::Path>,
        append: bool,
    ) -> std::io::Result<()> {
        self.core
            .macros
            .read_from_file(path, append, self.core.settings.vCodeTable as usize)
    }
    pub fn macros(&self) -> &macros::Macros {
        &self.core.macros
    }
    /// Reload macro outputs after a code-table change, preserving temporary spelling state.
    pub fn on_table_code_change(&mut self) {
        self.core
            .macros
            .on_table_code_change(self.core.settings.vCodeTable as usize);
    }
    pub fn delete_macro(&mut self, key: &str) -> bool {
        self.core
            .macros
            .delete(key, self.core.settings.vCodeTable as usize)
    }
    pub fn find_macro(&mut self, key: &mut Vec<u32>) -> Option<Vec<u32>> {
        let mut packed: Vec<i64> = key
            .iter()
            .map(|v| self.core.getCharacterCode(*v as i64))
            .collect();
        let result = self.core.macros.find(
            &mut packed,
            self.core.settings.vCodeTable as usize,
            self.core.settings.vAutoCapsMacro != 0,
        );
        *key = packed.into_iter().map(|v| v as u32).collect();
        result.map(|v| v.into_iter().map(|c| c as u32).collect())
    }
    pub(crate) fn set_macro_output(&mut self, key: Vec<u32>, data: Vec<u32>) {
        self.core.raw.macroKey = key.into_iter().map(|v| v as i64).collect();
        self.core.raw.macroData = data.into_iter().map(|v| v as i64).collect();
        self.sync_hook();
    }
    fn sync_hook(&mut self) -> &HookState {
        let r = &self.core.raw;
        self.hook.code = r.code as u8;
        self.hook.backspace_count = r.backspaceCount as u8;
        self.hook.new_char_count = r.newCharCount as u8;
        self.hook.ext_code = r.extCode as u8;
        for (d, s) in self.hook.char_data.iter_mut().zip(&r.charData) {
            *d = *s as u32;
        }
        self.hook.macro_key.clear();
        self.hook
            .macro_key
            .extend(r.macroKey.iter().map(|&c| c as u32));
        self.hook.macro_data.clear();
        self.hook
            .macro_data
            .extend(r.macroData.iter().map(|&c| c as u32));
        &self.hook
    }
}
impl core::Core {
    fn find_hook_macro(&mut self) -> i64 {
        for i in 0..self.raw.macroKey.len() {
            self.raw.macroKey[i] = self.getCharacterCode(self.raw.macroKey[i]);
        }
        match self.macros.find(
            &mut self.raw.macroKey,
            self.settings.vCodeTable as usize,
            self.settings.vAutoCapsMacro != 0,
        ) {
            Some(data) => {
                self.raw.macroData = data;
                1
            }
            None => 0,
        }
    }
}
