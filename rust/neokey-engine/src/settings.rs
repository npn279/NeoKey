#![allow(non_snake_case)]
/// Settings corresponding one-for-one to the original v-prefixed globals.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub vLanguage: i64,
    pub vInputType: i64,
    pub vFreeMark: i64,
    pub vCodeTable: i64,
    pub vSwitchKeyStatus: i64,
    pub vCheckSpelling: i64,
    pub vUseModernOrthography: i64,
    pub vQuickTelex: i64,
    pub vRestoreIfWrongSpelling: i64,
    pub vFixRecommendBrowser: i64,
    pub vUseMacro: i64,
    pub vUseMacroInEnglishMode: i64,
    pub vAutoCapsMacro: i64,
    pub vUseSmartSwitchKey: i64,
    pub vUpperCaseFirstChar: i64,
    pub vTempOffSpelling: i64,
    pub vAllowConsonantZFWJ: i64,
    pub vQuickStartConsonant: i64,
    pub vQuickEndConsonant: i64,
    pub vRememberCode: i64,
    pub vOtherLanguage: i64,
    pub vTempOffOpenKey: i64,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            vLanguage: 1,
            vInputType: 0,
            vFreeMark: 1,
            vCodeTable: 0,
            vSwitchKeyStatus: 0,
            vCheckSpelling: 1,
            vUseModernOrthography: 1,
            vQuickTelex: 0,
            vRestoreIfWrongSpelling: 1,
            vFixRecommendBrowser: 0,
            vUseMacro: 0,
            vUseMacroInEnglishMode: 0,
            vAutoCapsMacro: 0,
            vUseSmartSwitchKey: 0,
            vUpperCaseFirstChar: 0,
            vTempOffSpelling: 0,
            vAllowConsonantZFWJ: 0,
            vQuickStartConsonant: 0,
            vQuickEndConsonant: 0,
            vRememberCode: 0,
            vOtherLanguage: 0,
            vTempOffOpenKey: 0,
        }
    }
}
