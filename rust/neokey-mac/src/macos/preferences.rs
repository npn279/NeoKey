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
            vFreeMark: setting!("FreeMark", 0),
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

    pub fn get(&self, key: &NSString) -> i64 {
        self.defaults.integerForKey(key) as i64
    }

    pub fn bool(&self, key: &NSString) -> bool {
        self.defaults.boolForKey(key)
    }

    pub fn value_or(&self, key: &NSString, fallback: bool) -> bool {
        self.defaults
            .objectForKey(key)
            .map_or(fallback, |_| self.bool(key))
    }

    pub fn reset_typing(&self) {
        for key in TYPING_KEYS {
            self.defaults.removeObjectForKey(&NSString::from_str(key));
        }
    }

    pub fn integer(&self, key: &NSString, value: i64) {
        self.defaults.setInteger_forKey(value as isize, key);
    }

    pub fn synchronize(&self) {
        // The replacement process reads these preferences as soon as it starts.
        if !self.defaults.synchronize() {
            eprintln!("Could not synchronize NeoKey preferences before relaunch");
        }
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
    use objc2::AnyThread;

    #[test]
    fn preferences_use_standard_defaults_without_an_app_bundle() {
        objc2::rc::autoreleasepool(|_| {
            let (preferences, _) = Preferences::load();
            let standard = NSUserDefaults::standardUserDefaults();
            assert!(std::ptr::eq(&*preferences.defaults, &*standard));
        });
    }

    #[test]
    fn migration_imports_legacy_data_once_and_keeps_rust_values() {
        use objc2_foundation::NSNumber;

        objc2::rc::autoreleasepool(|_| {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let old_name = NSString::from_str(&format!(
                "com.neokey.test.legacy.{}.{nonce}",
                std::process::id()
            ));
            let new_name = NSString::from_str(&format!(
                "com.neokey.test.rust.{}.{nonce}",
                std::process::id()
            ));
            let old_store =
                NSUserDefaults::initWithSuiteName(NSUserDefaults::alloc(), Some(&old_name))
                    .unwrap();
            let new_store =
                NSUserDefaults::initWithSuiteName(NSUserDefaults::alloc(), Some(&new_name))
                    .unwrap();
            let defaults = NSUserDefaults::standardUserDefaults();
            let legacy_macros = NSData::with_bytes(&[5, 0, 42]);
            let legacy_apps = NSData::with_bytes(&[7, 0, 99]);
            old_store.setInteger_forKey(3, ns_string!("InputType"));
            old_store.setInteger_forKey(1, ns_string!("FreeMark"));
            old_store.setInteger_forKey(0, ns_string!("RunOnStartup"));
            old_store.setInteger_forKey(123, ns_string!("UnrelatedKey"));
            unsafe {
                old_store.setObject_forKey(Some(&legacy_macros), ns_string!("macroData"));
                old_store.setObject_forKey(Some(&legacy_apps), ns_string!("smartSwitchKey"));
            }
            new_store.setInteger_forKey(2, ns_string!("InputType"));
            old_store.synchronize();
            new_store.synchronize();
            let before = defaults.persistentDomainForName(&old_name).unwrap();
            let login = migrate_domain(&defaults, &old_name, &new_name);
            let migrated = defaults.persistentDomainForName(&new_name).unwrap();
            let retry = migrate_domain(&defaults, &old_name, &new_name);
            let after = defaults.persistentDomainForName(&old_name).unwrap();
            defaults.removePersistentDomainForName(&old_name);
            defaults.removePersistentDomainForName(&new_name);

            assert_eq!(login, Some(false));
            assert_eq!(retry, None);
            assert_eq!(before, after);
            let value = |key: &NSString| migrated.objectForKey(key).unwrap();
            assert_eq!(
                value(ns_string!("InputType"))
                    .downcast::<NSNumber>()
                    .unwrap()
                    .integerValue(),
                2
            );
            assert_eq!(
                value(ns_string!("FreeMark"))
                    .downcast::<NSNumber>()
                    .unwrap()
                    .integerValue(),
                1
            );
            assert_eq!(
                value(ns_string!("macroData")).downcast::<NSData>().unwrap(),
                legacy_macros
            );
            assert_eq!(
                value(ns_string!("smartSwitchKey"))
                    .downcast::<NSData>()
                    .unwrap(),
                legacy_apps
            );
            assert!(migrated.objectForKey(ns_string!("UnrelatedKey")).is_none());
        });
    }
}

const TYPING_KEYS: &[&str] = &[
    "SwitchKeyStatus",
    "InputMethod",
    "InputType",
    "FreeMark",
    "CodeTable",
    "Spelling",
    "ModernOrthography",
    "QuickTelex",
    "RestoreIfInvalidWord",
    "FixRecommendBrowser",
    "UseMacro",
    "UseMacroInEnglishMode",
    "vAutoCapsMacro",
    "UseSmartSwitchKey",
    "UpperCaseFirstChar",
    "vTempOffSpelling",
    "vAllowConsonantZFWJ",
    "vQuickStartConsonant",
    "vQuickEndConsonant",
    "vRememberCode",
    "vOtherLanguage",
    "vTempOffOpenKey",
    "SendKeyStepByStep",
    "vPerformLayoutCompat",
    "vFixChromiumBrowser",
    "ShowUIOnStartup",
    "GrayIcon",
    "vShowIconOnDock",
];

const OTHER_IMPORT_KEYS: &[&str] = &[
    "RunOnStartup",
    "DontCheckUpdate",
    "macroData",
    "smartSwitchKey",
    "convertToolDontAlertWhenCompleted",
    "convertToolToAllCaps",
    "convertToolToAllNonCaps",
    "convertToolToCapsFirstLetter",
    "convertToolToCapsEachWord",
    "convertToolRemoveMark",
    "convertToolFromCode",
    "convertToolToCode",
    "convertToolHotKey",
];

pub fn migrate_legacy_defaults() -> Option<bool> {
    use objc2_foundation::NSBundle;

    let current = ns_string!("com.neokey.rust");
    if NSBundle::mainBundle().bundleIdentifier().as_deref() != Some(current) {
        return None;
    }
    migrate_domain(
        &NSUserDefaults::standardUserDefaults(),
        ns_string!("com.tuyenmai.openkey"),
        current,
    )
}

fn migrate_domain(
    defaults: &NSUserDefaults,
    old_name: &NSString,
    new_name: &NSString,
) -> Option<bool> {
    use objc2::runtime::ProtocolObject;
    use objc2_foundation::NSMutableDictionary;

    let marker = ns_string!("DidImportOpenKeyDefaults");
    let current = defaults.persistentDomainForName(new_name);
    if current
        .as_ref()
        .is_some_and(|domain| domain.objectForKey(marker).is_some())
    {
        return None;
    }
    let first_launch = current.as_ref().is_none_or(|domain| domain.count() == 0);

    let merged = current.map_or_else(NSMutableDictionary::new, |domain| {
        use objc2_foundation::NSMutableCopying;
        domain.mutableCopy()
    });
    if let Some(old) = defaults.persistentDomainForName(old_name) {
        for key in TYPING_KEYS.iter().chain(OTHER_IMPORT_KEYS) {
            let key = NSString::from_str(key);
            if merged.objectForKey(&key).is_none() {
                if let Some(value) = old.objectForKey(&key) {
                    // The source is a persistent property-list domain.
                    unsafe { merged.setObject_forKey(&value, ProtocolObject::from_ref(&*key)) };
                }
            }
        }
    }
    let login_at_startup = merged
        .objectForKey(ns_string!("RunOnStartup"))
        .and_then(|value| value.downcast::<objc2_foundation::NSNumber>().ok())
        .map(|value| value.boolValue())
        .or(first_launch.then_some(true));
    let done = objc2_foundation::NSNumber::numberWithBool(true);
    unsafe {
        merged.setObject_forKey(&done, ProtocolObject::from_ref(marker));
        defaults.setPersistentDomain_forName(&merged, new_name);
    }
    login_at_startup
}
