// Faithful state-machine port of Sources/OpenKey/engine/Engine.cpp (GPL-3.0).
// Names and integer booleans follow upstream for source-level auditing. All state is per Engine.
#![allow(
    non_snake_case,
    unused_parens,
    unused_braces,
    unused_mut,
    unused_variables,
    unused_must_use
)]
use crate::{data::*, vietnamese::*, Settings};
mod composition;
mod events;
mod marks;
mod session;
mod spelling;
#[derive(Clone, Default)]
pub(crate) struct RawHook {
    pub code: i64,
    pub backspaceCount: i64,
    pub newCharCount: i64,
    pub extCode: i64,
    pub charData: [i64; 32],
    pub macroKey: Vec<i64>,
    pub macroData: Vec<i64>,
}
#[derive(Clone)]
pub(crate) struct Core {
    pub settings: Settings,
    pub raw: RawHook,
    pub TypingWord: [i64; 32],
    pub KeyStates: [i64; 32],
    _hasHandleQuickConsonant: i64,
    _hasHandledMacro: i64,
    _index: i64,
    _isCaps: i64,
    _isCharKeyCode: i64,
    _spaceCount: i64,
    _stateIndex: i64,
    _upperCaseStatus: i64,
    _useSpellCheckingBefore: i64,
    _willTempOffEngine: i64,
    i: i64,
    ii: i64,
    iii: i64,
    isChanged: i64,
    isCheckedGrammar: i64,
    isCorect: i64,
    isRestoredW: i64,
    j: i64,
    k: i64,
    keyForAEO: i64,
    kk: i64,
    l: i64,
    tempDisableKey: i64,
    vowelCount: i64,
    vowelEndIndex: i64,
    vowelStartIndex: i64,
    vowelWillSetMark: i64,
    _longWordHelper: Vec<i64>,
    _specialChar: Vec<i64>,
    _typingStates: Vec<Vec<i64>>,
    _typingStatesData: Vec<i64>,
    pub macros: crate::macros::Macros,
}
impl Core {
    pub(crate) fn initialize(&mut self) {
        self._index = 0;
        self._stateIndex = 0;
        self._useSpellCheckingBefore = self.settings.vCheckSpelling;
        self._typingStatesData.clear();
        self._typingStates.clear();
        self._longWordHelper.clear();
    }

    pub fn new(settings: Settings) -> Self {
        let check = settings.vCheckSpelling;
        Self {
            settings,
            raw: RawHook::default(),
            TypingWord: [0; 32],
            KeyStates: [0; 32],
            macros: Default::default(),
            _hasHandleQuickConsonant: 0,
            _hasHandledMacro: 0,
            _index: 0,
            _isCaps: 0,
            _isCharKeyCode: 0,
            _spaceCount: 0,
            _stateIndex: 0,
            _upperCaseStatus: 0,
            _useSpellCheckingBefore: check,
            _willTempOffEngine: 0,
            i: 0,
            ii: 0,
            iii: 0,
            isChanged: 0,
            isCheckedGrammar: 0,
            isCorect: 0,
            isRestoredW: 0,
            j: 0,
            k: 0,
            keyForAEO: 0,
            kk: 0,
            l: 0,
            tempDisableKey: 0,
            vowelCount: 0,
            vowelEndIndex: 0,
            vowelStartIndex: 0,
            vowelWillSetMark: 0,
            _longWordHelper: Vec::new(),
            _specialChar: Vec::new(),
            _typingStates: Vec::new(),
            _typingStatesData: Vec::new(),
        }
    }

    fn chr(&self, index: i64) -> i64 {
        self.TypingWord.get(index as usize).copied().unwrap_or(0) & CHAR_MASK
    }
    fn IS_CONSONANT(&self, k: i64) -> i64 {
        i64::from(![KEY_A, KEY_E, KEY_U, KEY_Y, KEY_I, KEY_O].contains(&k))
    }
    fn IS_BRACKET_KEY(&self, k: i64) -> i64 {
        i64::from([KEY_LEFT_BRACKET, KEY_RIGHT_BRACKET].contains(&k))
    }
    fn IS_NUMBER_KEY(&self, k: i64) -> i64 {
        i64::from(
            [
                KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_7, KEY_8, KEY_9, KEY_0,
            ]
            .contains(&k),
        )
    }
    fn IS_QUICK_TELEX_KEY(&self, k: i64) -> i64 {
        i64::from(
            self._index > 0
                && [KEY_C, KEY_G, KEY_K, KEY_N, KEY_Q, KEY_P, KEY_T].contains(&k)
                && self.chr(self._index - 1) == k,
        )
    }
    fn IS_SPECIALKEY(&self, k: i64) -> i64 {
        if self.settings.vInputType == vVNI {
            return self.IS_NUMBER_KEY(k);
        }
        i64::from(
            [
                KEY_W, KEY_E, KEY_R, KEY_O, KEY_A, KEY_S, KEY_D, KEY_F, KEY_J, KEY_Z, KEY_X,
            ]
            .contains(&k)
                || (self.settings.vInputType == vTelex && self.IS_BRACKET_KEY(k) != 0),
        )
    }
    fn IS_MARK_KEY(&self, k: i64) -> i64 {
        i64::from(if self.settings.vInputType == vVNI {
            [KEY_1, KEY_2, KEY_3, KEY_4, KEY_5].contains(&k)
        } else {
            [KEY_S, KEY_F, KEY_R, KEY_X, KEY_J].contains(&k)
        })
    }
    fn IS_KEY_W(&self, k: i64) -> i64 {
        i64::from(if self.settings.vInputType == vVNI {
            k == KEY_7 || k == KEY_8
        } else {
            k == KEY_W
        })
    }
    fn IS_KEY_DOUBLE(&self, k: i64) -> i64 {
        i64::from(if self.settings.vInputType == vVNI {
            k == KEY_6
        } else {
            [KEY_A, KEY_O, KEY_E].contains(&k)
        })
    }
    fn IS_KEY_S(&self, k: i64) -> i64 {
        i64::from(ProcessingChar[self.settings.vInputType as usize][0] == k)
    }
    fn IS_KEY_F(&self, k: i64) -> i64 {
        i64::from(ProcessingChar[self.settings.vInputType as usize][1] == k)
    }
    fn IS_KEY_R(&self, k: i64) -> i64 {
        i64::from(ProcessingChar[self.settings.vInputType as usize][2] == k)
    }
    fn IS_KEY_X(&self, k: i64) -> i64 {
        i64::from(ProcessingChar[self.settings.vInputType as usize][3] == k)
    }
    fn IS_KEY_J(&self, k: i64) -> i64 {
        i64::from(ProcessingChar[self.settings.vInputType as usize][4] == k)
    }
    fn IS_KEY_D(&self, k: i64) -> i64 {
        i64::from(ProcessingChar[self.settings.vInputType as usize][9] == k)
    }
    fn IS_KEY_Z(&self, k: i64) -> i64 {
        i64::from(ProcessingChar[self.settings.vInputType as usize][10] == k)
    }
}
