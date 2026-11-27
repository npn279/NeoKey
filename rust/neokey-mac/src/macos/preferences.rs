use neokey_engine::Settings;
use neokey_mac::typing::DEFAULT_HOTKEY;
use objc2::rc::Retained;
use objc2_foundation::{ns_string, NSData, NSString, NSUserDefaults};

pub struct Preferences {
    defaults: Retained<NSUserDefaults>,
    pub step_by_step: bool,
    pub layout_compat: bool,
    pub chromium_fix: bool,
}

impl Preferences {
    pub fn load() -> (Self, Settings) {
        // Foundation uses the bundle's domain, or the executable's default domain
        // for a bare binary. Passing the current bundle ID as a suite is invalid.
        let defaults = NSUserDefaults::standardUserDefaults();
        let integer = |key: &NSString, fallback| {
            if defaults.objectForKey(key).is_some() {
                defaults.integerForKey(key) as i64
            } else {
                fallback
            }
        };
        macro_rules! setting {
            ($key:literal, $fallback:expr) => {
                integer(ns_string!($key), $fallback)
            };
        }
        let hotkey = setting!("SwitchKeyStatus", 0);
        let settings = Settings {
            vLanguage: i64::from(setting!("InputMethod", 1) != 0),
            vInputType: setting!("InputType", 0).clamp(0, 3),
            vFreeMark: 0, // The macOS bridge deliberately disables FreeMark.
            vCodeTable: setting!("CodeTable", 0).clamp(0, 4),
            vSwitchKeyStatus: if hotkey == 0 {
                i64::from(DEFAULT_HOTKEY)
            } else {
                hotkey
            },
            vCheckSpelling: setting!("Spelling", 1),
            vUseModernOrthography: setting!("ModernOrthography", 0),
            vQuickTelex: setting!("QuickTelex", 0),
            vRestoreIfWrongSpelling: setting!("RestoreIfInvalidWord", 0),
            vFixRecommendBrowser: setting!("FixRecommendBrowser", 1),
            vUseMacro: setting!("UseMacro", 1),
            vUseMacroInEnglishMode: setting!("UseMacroInEnglishMode", 0),
            vAutoCapsMacro: setting!("vAutoCapsMacro", 0),
            vUseSmartSwitchKey: setting!("UseSmartSwitchKey", 1),
            vUpperCaseFirstChar: setting!("UpperCaseFirstChar", 0),
            vTempOffSpelling: setting!("vTempOffSpelling", 0),
            vAllowConsonantZFWJ: setting!("vAllowConsonantZFWJ", 0),
            vQuickStartConsonant: setting!("vQuickStartConsonant", 0),
            vQuickEndConsonant: setting!("vQuickEndConsonant", 0),
            vRememberCode: setting!("vRememberCode", 1),
            vOtherLanguage: setting!("vOtherLanguage", 1),
            vTempOffOpenKey: setting!("vTempOffOpenKey", 0),
        };
        let preferences = Self {
            step_by_step: setting!("SendKeyStepByStep", 0) != 0,
            layout_compat: setting!("vPerformLayoutCompat", 0) != 0,
            chromium_fix: setting!("vFixChromiumBrowser", 0) != 0,
            defaults,
        };
        (preferences, settings)
    }

    pub fn integer(&self, key: &NSString, value: i64) {
        self.defaults.setInteger_forKey(value as isize, key);
    }

    pub fn data(&self, key: &NSString) -> Vec<u8> {
        self.defaults
            .dataForKey(key)
            .map_or_else(Vec::new, |data| data.to_vec())
    }

    pub fn set_data(&self, key: &NSString, bytes: &[u8]) {
        // NSData is a property-list value, as required by NSUserDefaults.
        unsafe {
            self.defaults
                .setObject_forKey(Some(&NSData::with_bytes(bytes)), key)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferences_use_standard_defaults_without_an_app_bundle() {
        objc2::rc::autoreleasepool(|_| {
            let (preferences, _) = Preferences::load();
            let standard = NSUserDefaults::standardUserDefaults();
            assert!(std::ptr::eq(&*preferences.defaults, &*standard));
        });
    }
}
