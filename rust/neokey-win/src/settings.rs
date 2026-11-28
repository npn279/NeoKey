//! Registry schema shared with OpenKey's Win32 app.
use neokey_engine::{ConvertOptions, Settings};

pub const REGISTRY_PATH: &str = r"SOFTWARE\TuyenMai\OpenKey";
pub const RUN_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
pub const RUN_VALUE: &str = "OpenKey";
pub const MACRO_DATA: &str = "macroData";
pub const SMART_SWITCH_DATA: &str = "smartSwitchKey";

pub const INPUT_TYPES: &[&str] = &["Telex", "VNI", "Simple Telex", "Simple Telex 2"];
pub const CODE_TABLES: &[&str] = &[
    "Unicode dựng sẵn",
    "TCVN3 (ABC)",
    "VNI Windows",
    "Unicode tổ hợp",
    "Vietnamese Locale CP 1258",
];

macro_rules! registry_settings {
    ($($name:ident, $key:ident, $default:expr);+ $(;)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        #[repr(usize)]
        pub enum Setting { $($name),+ }

        impl Setting {
            pub const ALL: &[Self] = &[$(Self::$name),+];
            pub const fn key(self) -> &'static str {
                match self { $(Self::$name => stringify!($key)),+ }
            }
            const fn default_value(self) -> u32 {
                match self { $(Self::$name => $default),+ }
            }
        }
    };
}

registry_settings! {
    Language, vLanguage, 1;
    InputType, vInputType, 0;
    CodeTable, vCodeTable, 0;
    Spelling, vCheckSpelling, 1;
    ModernOrthography, vUseModernOrthography, 0;
    QuickTelex, vQuickTelex, 0;
    SwitchKey, vSwitchKeyStatus, 0x5a00_025a;
    RestoreSpelling, vRestoreIfWrongSpelling, 1;
    BrowserFix, vFixRecommendBrowser, 1;
    UseMacro, vUseMacro, 1;
    EnglishMacro, vUseMacroInEnglishMode, 0;
    MacroCaps, vAutoCapsMacro, 0;
    StepByStep, vSendKeyStepByStep, 1;
    GrayIcon, vUseGrayIcon, 0;
    ShowAtStartup, vShowOnStartUp, 1;
    RunWithWindows, vRunWithWindows, 1;
    SmartSwitch, vUseSmartSwitchKey, 1;
    FirstLetterCaps, vUpperCaseFirstChar, 0;
    ForeignConsonants, vAllowConsonantZFWJ, 0;
    TemporarySpelling, vTempOffSpelling, 0;
    QuickStart, vQuickStartConsonant, 0;
    QuickEnd, vQuickEndConsonant, 0;
    Metro, vSupportMetroApp, 0;
    Administrator, vRunAsAdmin, 0;
    DesktopShortcut, vCreateDesktopShortcut, 0;
    CheckUpdate, vCheckNewVersion, 0;
    RememberTable, vRememberCode, 1;
    OtherLanguages, vOtherLanguage, 1;
    TemporaryEngine, vTempOffOpenKey, 0;
    ChromiumFix, vFixChromiumBrowser, 0;
    QuietConvert, convertToolDontAlertWhenCompleted, 0;
    AllCaps, convertToolToAllCaps, 0;
    AllLower, convertToolToAllNonCaps, 0;
    SentenceCaps, convertToolToCapsFirstLetter, 0;
    WordCaps, convertToolToCapsEachWord, 0;
    RemoveMarks, convertToolRemoveMark, 0;
    FromTable, convertToolFromCode, 0;
    ToTable, convertToolToCode, 0;
    ConvertKey, convertToolHotKey, Hotkey::EMPTY.0;
}

#[derive(Clone, Debug)]
pub struct Preferences([u32; Setting::ALL.len()]);

impl Default for Preferences {
    fn default() -> Self {
        Self::load(|_| None)
    }
}

impl Preferences {
    pub fn load(mut read: impl FnMut(&str) -> Option<u32>) -> Self {
        let mut preferences = Self([0; Setting::ALL.len()]);
        for &setting in Setting::ALL {
            preferences.set(
                setting,
                read(setting.key()).unwrap_or_else(|| setting.default_value()),
            );
        }
        preferences
    }

    pub fn get(&self, setting: Setting) -> u32 {
        self.0[setting as usize]
    }

    pub fn enabled(&self, setting: Setting) -> bool {
        self.get(setting) != 0
    }

    pub fn set(&mut self, setting: Setting, value: u32) {
        use Setting::*;
        self.0[setting as usize] = match setting {
            InputType if value >= INPUT_TYPES.len() as u32 => 0,
            CodeTable | FromTable | ToTable if value >= CODE_TABLES.len() as u32 => 0,
            ConvertKey if value == 0 => Hotkey::EMPTY.0,
            // The original initializer mixed the Mac Z keycode and an ASCII label.
            SwitchKey if value == 0x7a00_0206 => SwitchKey.default_value(),
            SwitchKey | ConvertKey | CodeTable | FromTable | ToTable | InputType => value,
            _ => u32::from(value != 0),
        }
    }

    pub fn reset_main(&mut self) {
        use Setting::*;
        // The original reset leaves conversion, auto-cap macros and system actions alone.
        for &setting in &Setting::ALL[..=ChromiumFix as usize] {
            match setting {
                // Original reset leaves these, plus the quick-consonant toggles, unchanged.
                MacroCaps | Administrator | DesktopShortcut | CheckUpdate | QuickStart
                | QuickEnd => {}
                UseMacro => self.set(setting, 0),
                Metro => self.set(setting, 1),
                _ => self.set(setting, setting.default_value()),
            }
        }
    }

    pub fn engine(&self) -> Settings {
        use Setting::*;
        let value = |setting| i64::from(self.get(setting));
        Settings {
            vLanguage: value(Language),
            vInputType: value(InputType),
            vFreeMark: 0,
            vCodeTable: value(CodeTable),
            vSwitchKeyStatus: value(SwitchKey),
            vCheckSpelling: value(Spelling),
            vUseModernOrthography: value(ModernOrthography),
            vQuickTelex: value(QuickTelex),
            vRestoreIfWrongSpelling: value(RestoreSpelling),
            vFixRecommendBrowser: value(BrowserFix),
            vUseMacro: value(UseMacro),
            vUseMacroInEnglishMode: value(EnglishMacro),
            vAutoCapsMacro: value(MacroCaps),
            vUseSmartSwitchKey: value(SmartSwitch),
            vUpperCaseFirstChar: value(FirstLetterCaps),
            vTempOffSpelling: value(TemporarySpelling),
            vAllowConsonantZFWJ: value(ForeignConsonants),
            vQuickStartConsonant: value(QuickStart),
            vQuickEndConsonant: value(QuickEnd),
            vRememberCode: value(RememberTable),
            vOtherLanguage: value(OtherLanguages),
            vTempOffOpenKey: value(TemporaryEngine),
        }
    }

    pub fn convert(&self) -> ConvertOptions {
        use Setting::*;
        ConvertOptions {
            dont_alert_when_completed: self.enabled(QuietConvert),
            to_all_caps: self.enabled(AllCaps),
            to_all_non_caps: self.enabled(AllLower),
            to_caps_first_letter: self.enabled(SentenceCaps),
            to_caps_each_word: self.enabled(WordCaps),
            remove_mark: self.enabled(RemoveMarks),
            from_code: self.get(FromTable) as u8,
            to_code: self.get(ToTable) as u8,
            hot_key: self.get(ConvertKey) as i32,
        }
    }
}

pub const CTRL: u8 = 1;
pub const ALT: u8 = 2;
pub const WIN: u8 = 4;
pub const SHIFT: u8 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hotkey(pub u32);

impl Hotkey {
    pub const MODIFIERS_ONLY: u8 = 0xfe;
    pub const EMPTY: Self = Self(0xfe00_00fe);
    const BEEP: u32 = 0x8000;

    pub fn vk(self) -> u8 {
        self.0 as u8
    }

    pub fn mods(self) -> u8 {
        (self.0 >> 8) as u8 & 0xf
    }

    pub fn beep(self) -> bool {
        self.0 & Self::BEEP != 0
    }

    pub fn matches(self, vk: u16, mods: u8) -> bool {
        self.0 != 0
            && self.0 & !Self::BEEP != Self::EMPTY.0
            && u16::from(self.vk()) == vk
            && self.mods() == mods
    }

    pub fn with_key(self, vk: u8) -> Self {
        Self(self.0 & 0x00ff_ff00 | u32::from(vk) << 24 | u32::from(vk))
    }

    pub fn with_mods(self, mods: u8) -> Self {
        Self(self.0 & !0x0f00 | u32::from(mods & 0xf) << 8)
    }

    pub fn with_beep(self, beep: bool) -> Self {
        Self(self.0 & !Self::BEEP | if beep { Self::BEEP } else { 0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_windows_names_and_keeps_legacy_dword_encoding() {
        let preferences = Preferences::load(|key| match key {
            "vLanguage" => Some(0),
            "vCodeTable" => Some(2),
            "vSwitchKeyStatus" => Some(0xfe00_86fe),
            "convertToolFromCode" => Some(4),
            "convertToolToCode" => Some(u32::MAX),
            _ => None,
        });
        assert_eq!(preferences.engine().vLanguage, 0);
        assert_eq!(preferences.engine().vCodeTable, 2);
        assert_eq!(preferences.convert().from_code, 4);
        assert_eq!(preferences.convert().to_code, 0);
        let shortcut = Hotkey(preferences.get(Setting::SwitchKey));
        assert!(shortcut.beep());
        assert!(shortcut.matches(0xfe, ALT | WIN));
        assert!(!shortcut.matches(0xfe, ALT | WIN | SHIFT));
        assert_eq!(shortcut.with_key(b'Z').0, 0x5a00_865a);
        assert!(!Hotkey::EMPTY.with_beep(true).matches(0xfe, 0));
    }

    #[test]
    fn original_broken_first_run_shortcut_is_loaded_as_alt_z() {
        let preferences = Preferences::load(|_| Some(0x7a00_0206));
        assert!(Hotkey(preferences.get(Setting::SwitchKey)).matches(b'Z'.into(), ALT));
    }

    #[test]
    fn reset_keeps_quick_consonants_and_the_actions_the_original_dialog_left_alone() {
        let mut preferences = Preferences::default();
        preferences.set(Setting::QuickStart, 1);
        preferences.set(Setting::QuickEnd, 1);
        preferences.set(Setting::MacroCaps, 1);
        preferences.set(Setting::UseMacro, 1);
        preferences.set(Setting::Metro, 0);
        preferences.set(Setting::Administrator, 1);
        preferences.set(Setting::DesktopShortcut, 1);
        preferences.set(Setting::CheckUpdate, 1);
        preferences.reset_main();
        assert!(preferences.enabled(Setting::QuickStart));
        assert!(preferences.enabled(Setting::QuickEnd));
        assert!(preferences.enabled(Setting::MacroCaps));
        assert!(preferences.enabled(Setting::Administrator));
        assert!(preferences.enabled(Setting::DesktopShortcut));
        assert!(preferences.enabled(Setting::CheckUpdate));
        assert!(!preferences.enabled(Setting::UseMacro));
        assert!(preferences.enabled(Setting::Metro));
        assert!(preferences.enabled(Setting::RunWithWindows));
    }
}
