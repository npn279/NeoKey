// Ported from Engine.cpp; preserve upstream scratch-counter and packed-output semantics.
use super::*;
impl Core {
    // Port of Engine.cpp::setKeyData.
    pub(super) fn setKeyData(&mut self, index: i64, keyCode: i64, isCaps: i64) {
        if ((index < 0) || (index >= MAX_BUFF)) {
            return;
        }
        self.TypingWord[(index) as usize] =
            ((keyCode) | (if (isCaps != 0) { CAPS_MASK } else { 0 }));
    }
    // Port of Engine.cpp::insertKey.
    pub(super) fn insertKey(&mut self, keyCode: i64, isCaps: i64, isCheckSpelling: i64) {
        if (self._index >= MAX_BUFF) {
            self._longWordHelper.push(self.TypingWord[(0) as usize]);
            self.iii = 0;
            while (self.iii < ((MAX_BUFF) - (1))) {
                {
                    self.TypingWord[(self.iii) as usize] =
                        self.TypingWord[((self.iii) + (1)) as usize];
                }
                self.iii += 1;
            }
            self.setKeyData(((self._index) - (1)), keyCode, isCaps);
        } else {
            {
                let index = {
                    let old = self._index;
                    self._index += 1;
                    old
                };
                self.setKeyData(index, keyCode, isCaps);
            };
        }
        if ((self.settings.vCheckSpelling != 0) && (isCheckSpelling != 0)) {
            self.checkSpelling(0);
        }
        if (((keyCode == KEY_D) && (((self._index) - (2)) >= 0))
            && (self.IS_CONSONANT(self.chr(((self._index) - (2)))) != 0))
        {
            self.tempDisableKey = 0;
        }
    }
    // Port of Engine.cpp::insertState.
    pub(super) fn insertState(&mut self, keyCode: i64, isCaps: i64) {
        if (self._stateIndex >= MAX_BUFF) {
            self.iii = 0;
            while (self.iii < ((MAX_BUFF) - (1))) {
                {
                    self.KeyStates[(self.iii) as usize] =
                        self.KeyStates[((self.iii) + (1)) as usize];
                }
                self.iii += 1;
            }
            self.KeyStates[((self._stateIndex) - (1)) as usize] =
                ((keyCode) | (if (isCaps != 0) { CAPS_MASK } else { 0 }));
        } else {
            self.KeyStates[({
                let old = self._stateIndex;
                self._stateIndex += 1;
                old
            }) as usize] = ((keyCode) | (if (isCaps != 0) { CAPS_MASK } else { 0 }));
        }
    }
    // Port of Engine.cpp::saveWord.
    pub(super) fn saveWord(&mut self) {
        if (self.raw.code != vReplaceMaro) {
            if (self._index > 0) {
                if ((self._longWordHelper.len() as i64) > 0) {
                    self._typingStatesData.clear();
                    self.i = 0;
                    while (self.i < (self._longWordHelper.len() as i64)) {
                        {
                            if ((self.i != 0) && (((self.i) % (MAX_BUFF)) == 0)) {
                                self._typingStates.push(self._typingStatesData.clone());
                                self._typingStatesData.clear();
                            }
                            self._typingStatesData
                                .push(self._longWordHelper[(self.i) as usize]);
                        }
                        self.i += 1;
                    }
                    self._typingStates.push(self._typingStatesData.clone());
                    self._longWordHelper.clear();
                }
                self._typingStatesData.clear();
                self.i = 0;
                while (self.i < self._index) {
                    {
                        self._typingStatesData
                            .push(self.TypingWord[(self.i) as usize]);
                    }
                    self.i += 1;
                }
                self._typingStates.push(self._typingStatesData.clone());
            }
        } else {
            self._typingStatesData.clear();
            self.i = 0;
            while (self.i < (self.raw.macroData.len() as i64)) {
                {
                    if ((self.i != 0) && (((self.i) % (MAX_BUFF)) == 0)) {
                        self._typingStates.push(self._typingStatesData.clone());
                        self._typingStatesData.clear();
                    }
                    self._typingStatesData
                        .push(self.raw.macroData[(self.i) as usize]);
                }
                self.i += 1;
            }
            self._typingStates.push(self._typingStatesData.clone());
        }
    }
    // Port of Engine.cpp::saveSpaces.
    pub(super) fn saveSpaces(&mut self, keyCode: i64, count: i64) {
        self._typingStatesData.clear();
        self.i = 0;
        while (self.i < count) {
            {
                self._typingStatesData.push(keyCode);
            }
            self.i += 1;
        }
        self._typingStates.push(self._typingStatesData.clone());
    }
    // Port of Engine.cpp::saveSpecialChar.
    pub(super) fn saveSpecialChar(&mut self) {
        self._typingStatesData.clear();
        self.i = 0;
        while (self.i < (self._specialChar.len() as i64)) {
            {
                self._typingStatesData
                    .push(self._specialChar[(self.i) as usize]);
            }
            self.i += 1;
        }
        self._typingStates.push(self._typingStatesData.clone());
        self._specialChar.clear();
    }
    // Port of Engine.cpp::restoreLastTypingState.
    pub(super) fn restoreLastTypingState(&mut self) {
        if ((self._typingStates.len() as i64) > 0) {
            self._typingStatesData = self._typingStates.last().cloned().unwrap_or_default();
            self._typingStates.pop();
            if ((self._typingStatesData.len() as i64) > 0) {
                if (self._typingStatesData[(0) as usize] == KEY_SPACE) {
                    self._spaceCount = ((self._typingStatesData.len() as i64) as i32 as i64);
                    self._index = ((0) as u8 as i64);
                } else {
                    if key_classes(self._typingStatesData[0] as u16 as i64) & CHARACTER_KEY != 0 {
                        self._index = ((0) as u8 as i64);
                        self._specialChar = self._typingStatesData.clone();
                        self.checkSpelling(0);
                    } else {
                        self.i = 0;
                        while (self.i < (self._typingStatesData.len() as i64)) {
                            {
                                self.TypingWord[(self.i) as usize] =
                                    self._typingStatesData[(self.i) as usize];
                            }
                            self.i += 1;
                        }
                        self._index =
                            (((self._typingStatesData.len() as i64) as u8 as i64) as u8 as i64);
                    }
                }
            }
        }
    }
    // Port of Engine.cpp::startNewSession.
    pub(crate) fn startNewSession(&mut self) {
        self._index = ((0) as u8 as i64);
        self.raw.backspaceCount = ((0) as u8 as i64);
        self.raw.newCharCount = ((0) as u8 as i64);
        self.tempDisableKey = 0;
        self._stateIndex = ((0) as u8 as i64);
        self._hasHandledMacro = 0;
        self._hasHandleQuickConsonant = 0;
        self._longWordHelper.clear();
    }
    // Port of Engine.cpp::checkRestoreIfWrongSpelling.
    pub(super) fn checkRestoreIfWrongSpelling(&mut self, handleCode: i64) -> i64 {
        self.ii = 0;
        while (self.ii < self._index) {
            {
                if (!(self.IS_CONSONANT(self.chr(self.ii)) != 0)
                    && (((((self.TypingWord[(self.ii) as usize]) & (MARK_MASK)) != 0)
                        || (((self.TypingWord[(self.ii) as usize]) & (TONE_MASK)) != 0))
                        || (((self.TypingWord[(self.ii) as usize]) & (TONEW_MASK)) != 0)))
                {
                    self.raw.code = ((handleCode) as u8 as i64);
                    self.raw.backspaceCount = ((self._index) as u8 as i64);
                    self.raw.newCharCount = ((self._stateIndex) as u8 as i64);
                    self.i = 0;
                    while (self.i < self._stateIndex) {
                        {
                            self.TypingWord[(self.i) as usize] = self.KeyStates[(self.i) as usize];
                            self.raw.charData[(((self._stateIndex) - (1)) - (self.i)) as usize] =
                                self.TypingWord[(self.i) as usize];
                        }
                        self.i += 1;
                    }
                    self._index = ((self._stateIndex) as u8 as i64);
                    return 1;
                }
            }
            self.ii += 1;
        }
        return 0;
    }
    // Port of Engine.cpp::vTempOffSpellChecking.
    pub(crate) fn vTempOffSpellChecking(&mut self) {
        if (self._useSpellCheckingBefore != 0) {
            self.settings.vCheckSpelling = (if (self.settings.vCheckSpelling != 0) {
                0
            } else {
                1
            });
        }
    }
    // Port of Engine.cpp::vSetCheckSpelling.
    pub(crate) fn vSetCheckSpelling(&mut self) {
        self._useSpellCheckingBefore = self.settings.vCheckSpelling;
    }
    // Port of Engine.cpp::vTempOffEngine.
    pub(crate) fn vTempOffEngine(&mut self, off: i64) {
        self._willTempOffEngine = off;
    }
}
