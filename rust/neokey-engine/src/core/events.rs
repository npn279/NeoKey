// Ported from Engine.cpp; preserve upstream scratch-counter and packed-output semantics.
use super::*;
impl Core {
    // Port of Engine.cpp::vEnglishMode.
    pub(crate) fn vEnglishMode(
        &mut self,
        state: i64,
        data: i64,
        isCaps: i64,
        otherControlKey: i64,
    ) {
        self.raw.code = ((vDoNothing) as u8 as i64);
        if ((state == MouseDown) || ((otherControlKey != 0) && !(isCaps != 0))) {
            self.raw.macroKey.clear();
            self._willTempOffEngine = 0;
        } else {
            if (data == KEY_SPACE) {
                if (!(self._hasHandledMacro != 0) && (self.find_hook_macro() != 0)) {
                    self.raw.code = ((vReplaceMaro) as u8 as i64);
                    self.raw.backspaceCount =
                        (((self.raw.macroKey.len() as i64) as u8 as i64) as u8 as i64);
                }
                self.raw.macroKey.clear();
                self._willTempOffEngine = 0;
            } else {
                if (data == KEY_DELETE) {
                    if ((self.raw.macroKey.len() as i64) > 0) {
                        self.raw.macroKey.pop();
                    } else {
                        self._willTempOffEngine = 0;
                    }
                } else {
                    if key_classes(data) & (WORD_BREAK_KEY | CHARACTER_KEY) == WORD_BREAK_KEY {
                        self.raw.macroKey.clear();
                        self._willTempOffEngine = 0;
                    } else {
                        if !(self._willTempOffEngine != 0) {
                            self.raw
                                .macroKey
                                .push(((data) | (if (isCaps != 0) { CAPS_MASK } else { 0 })));
                        }
                    }
                }
            }
        }
    }
    // Port of Engine.cpp::vKeyHandleEvent.
    pub(crate) fn vKeyHandleEvent(
        &mut self,
        event: i64,
        state: i64,
        data: i64,
        capsStatus: i64,
        otherControlKey: i64,
    ) {
        let classes = key_classes(data);
        let word_break = event == Mouse || classes & WORD_BREAK_KEY != 0;
        let macro_break = classes & MACRO_BREAK_KEY != 0;
        self._isCaps = i64::from(
            ((i64::from((capsStatus) == (1))) != 0) || ((i64::from((capsStatus) == (2))) != 0),
        );
        if (((((self.IS_NUMBER_KEY(data) != 0) && (capsStatus == 1)) || (otherControlKey != 0))
            || word_break)
            || ((self._index == 0) && (self.IS_NUMBER_KEY(data) != 0)))
        {
            self.raw.code = ((vDoNothing) as u8 as i64);
            self.raw.backspaceCount = ((0) as u8 as i64);
            self.raw.newCharCount = ((0) as u8 as i64);
            self.raw.extCode = ((1) as u8 as i64);
            if ((((self.settings.vUseMacro != 0) && macro_break) && !(self._hasHandledMacro != 0))
                && (self.find_hook_macro() != 0))
            {
                self.raw.code = ((vReplaceMaro) as u8 as i64);
                self.raw.backspaceCount =
                    (((self.raw.macroKey.len() as i64) as u8 as i64) as u8 as i64);
                self._hasHandledMacro = 1;
            } else {
                if ((((self.settings.vQuickStartConsonant != 0)
                    || (self.settings.vQuickEndConsonant != 0))
                    && !(self.tempDisableKey != 0))
                    && macro_break)
                {
                    self.checkQuickConsonant();
                } else {
                    if ((self.settings.vRestoreIfWrongSpelling != 0) && word_break) {
                        if (!(self.tempDisableKey != 0) && (self.settings.vCheckSpelling != 0)) {
                            self.checkSpelling(1);
                        }
                        if ((self.tempDisableKey != 0)
                            && !(self.checkRestoreIfWrongSpelling(vRestoreAndStartNewSession) != 0))
                        {
                            self.raw.code = ((vDoNothing) as u8 as i64);
                        }
                    }
                }
            }
            self._isCharKeyCode = i64::from(
                ((i64::from((state) == (KeyDown))) != 0) && (classes & CHARACTER_KEY != 0),
            );
            if !(self._isCharKeyCode != 0) {
                self._specialChar.clear();
                self._typingStates.clear();
            } else {
                if (self._spaceCount > 0) {
                    self.saveSpaces(KEY_SPACE, self._spaceCount);
                    self._spaceCount = 0;
                } else {
                    self.saveWord();
                }
                self._specialChar
                    .push(((data) | (if (self._isCaps != 0) { CAPS_MASK } else { 0 })));
                self.raw.extCode = ((3) as u8 as i64);
            }
            if (self.raw.code == vDoNothing) {
                self.startNewSession();
                self.settings.vCheckSpelling = self._useSpellCheckingBefore;
                self._willTempOffEngine = 0;
            } else {
                if ((self.raw.code == vReplaceMaro) || (self._hasHandleQuickConsonant != 0)) {
                    self._index = ((0) as u8 as i64);
                }
            }
            if (self.settings.vUseMacro != 0) {
                if (self._isCharKeyCode != 0) {
                    self.raw
                        .macroKey
                        .push(((data) | (if (self._isCaps != 0) { CAPS_MASK } else { 0 })));
                } else {
                    self.raw.macroKey.clear();
                }
            }
            if (self.settings.vUpperCaseFirstChar != 0) {
                if (data == KEY_DOT) {
                    self._upperCaseStatus = ((1) as u8 as i64);
                } else {
                    if ((data == KEY_ENTER) || (data == KEY_RETURN)) {
                        self._upperCaseStatus = ((2) as u8 as i64);
                    } else {
                        self._upperCaseStatus = ((0) as u8 as i64);
                    }
                }
            }
        } else {
            if (data == KEY_SPACE) {
                if (!(self.tempDisableKey != 0) && (self.settings.vCheckSpelling != 0)) {
                    self.checkSpelling(1);
                }
                if (((self.settings.vUseMacro != 0) && !(self._hasHandledMacro != 0))
                    && (self.find_hook_macro() != 0))
                {
                    self.raw.code = ((vReplaceMaro) as u8 as i64);
                    self.raw.backspaceCount =
                        (((self.raw.macroKey.len() as i64) as u8 as i64) as u8 as i64);
                    self._spaceCount += 1;
                    self._hasHandledMacro = 1;
                } else {
                    if ((((self.settings.vQuickStartConsonant != 0)
                        || (self.settings.vQuickEndConsonant != 0))
                        && !(self.tempDisableKey != 0))
                        && (self.checkQuickConsonant() != 0))
                    {
                        self._spaceCount += 1;
                    } else {
                        if (((self.settings.vRestoreIfWrongSpelling != 0)
                            && (self.tempDisableKey != 0))
                            && !(self._hasHandledMacro != 0))
                        {
                            if !(self.checkRestoreIfWrongSpelling(vRestore) != 0) {
                                self.raw.code = ((vDoNothing) as u8 as i64);
                            }
                            self._spaceCount += 1;
                        } else {
                            self.raw.code = ((vDoNothing) as u8 as i64);
                            self._spaceCount += 1;
                        }
                    }
                }
                if (self.settings.vUseMacro != 0) {
                    self.raw.macroKey.clear();
                }
                if ((self.settings.vUpperCaseFirstChar != 0) && (self._upperCaseStatus == 1)) {
                    self._upperCaseStatus = ((2) as u8 as i64);
                }
                if (self._spaceCount == 1) {
                    if ((self._specialChar.len() as i64) > 0) {
                        self.saveSpecialChar();
                    } else {
                        self.saveWord();
                    }
                }
                self.settings.vCheckSpelling = self._useSpellCheckingBefore;
                self._willTempOffEngine = 0;
            } else {
                if (data == KEY_DELETE) {
                    self.raw.code = ((vDoNothing) as u8 as i64);
                    self.raw.extCode = ((2) as u8 as i64);
                    if ((self._specialChar.len() as i64) > 0) {
                        self._specialChar.pop();
                        if ((self._specialChar.len() as i64) == 0) {
                            self.restoreLastTypingState();
                        }
                    } else {
                        if (self._spaceCount > 0) {
                            self._spaceCount -= 1;
                            if (self._spaceCount == 0) {
                                self.restoreLastTypingState();
                            }
                        } else {
                            if (self._stateIndex > 0) {
                                self._stateIndex = ((self._stateIndex - 1) as u8 as i64);
                            }
                            if (self._index > 0) {
                                self._index = ((self._index - 1) as u8 as i64);
                                if ((self._longWordHelper.len() as i64) > 0) {
                                    self.i = ((MAX_BUFF) - (1));
                                    while (self.i > 0) {
                                        {
                                            self.TypingWord[(self.i) as usize] =
                                                self.TypingWord[((self.i) - (1)) as usize];
                                        }
                                        self.i -= 1;
                                    }
                                    self.TypingWord[(0) as usize] =
                                        self._longWordHelper.last().cloned().unwrap_or_default();
                                    self._longWordHelper.pop();
                                    self._index = ((self._index + 1) as u8 as i64);
                                }
                                if (self.settings.vCheckSpelling != 0) {
                                    self.checkSpelling(0);
                                }
                            }
                            if ((self.settings.vUseMacro != 0)
                                && ((self.raw.macroKey.len() as i64) > 0))
                            {
                                self.raw.macroKey.pop();
                            }
                            self.raw.backspaceCount = ((0) as u8 as i64);
                            self.raw.newCharCount = ((0) as u8 as i64);
                            self.raw.extCode = ((2) as u8 as i64);
                            if (self._index == 0) {
                                self.startNewSession();
                                self._specialChar.clear();
                                self.restoreLastTypingState();
                            } else {
                                self.checkGrammar(1);
                            }
                        }
                    }
                } else {
                    if (self._willTempOffEngine != 0) {
                        self.raw.code = ((vDoNothing) as u8 as i64);
                        self.raw.extCode = ((3) as u8 as i64);
                        return;
                    }
                    if (self._spaceCount > 0) {
                        self.raw.backspaceCount = ((0) as u8 as i64);
                        self.raw.newCharCount = ((0) as u8 as i64);
                        self.raw.extCode = ((0) as u8 as i64);
                        self.startNewSession();
                        self.saveSpaces(KEY_SPACE, self._spaceCount);
                        self._spaceCount = 0;
                    } else {
                        if ((self._specialChar.len() as i64) > 0) {
                            self.saveSpecialChar();
                        }
                    }
                    self.insertState(data, self._isCaps);
                    if (!(self.IS_SPECIALKEY(data) != 0) || (self.tempDisableKey != 0)) {
                        if ((self.settings.vQuickTelex != 0)
                            && (self.IS_QUICK_TELEX_KEY(data) != 0))
                        {
                            self.handleQuickTelex(data, self._isCaps);
                            return;
                        } else {
                            self.raw.code = ((vDoNothing) as u8 as i64);
                            self.raw.backspaceCount = ((0) as u8 as i64);
                            self.raw.newCharCount = ((0) as u8 as i64);
                            self.raw.extCode = ((3) as u8 as i64);
                            self.insertKey(data, self._isCaps, 1);
                        }
                    } else {
                        self.raw.code = ((vDoNothing) as u8 as i64);
                        self.raw.extCode = ((3) as u8 as i64);
                        self.handleMainKey(data, self._isCaps);
                    }
                    if (!(self.settings.vFreeMark != 0) && !(self.IS_KEY_D(data) != 0)) {
                        if (self.raw.code == vDoNothing) {
                            self.checkGrammar((-(1)));
                        } else {
                            self.checkGrammar(0);
                        }
                    }
                    if (self.raw.code == vRestore) {
                        self.insertKey(data, self._isCaps, 1);
                        self._stateIndex = ((self._stateIndex - 1) as u8 as i64);
                    }
                    if (self.settings.vUseMacro != 0) {
                        if (self.raw.code == vDoNothing) {
                            self.raw
                                .macroKey
                                .push(((data) | (if (self._isCaps != 0) { CAPS_MASK } else { 0 })));
                        } else {
                            if ((self.raw.code == vWillProcess) || (self.raw.code == vRestore)) {
                                self.i = 0;
                                while (self.i < self.raw.backspaceCount) {
                                    {
                                        if ((self.raw.macroKey.len() as i64) > 0) {
                                            self.raw.macroKey.pop();
                                        }
                                    }
                                    self.i += 1;
                                }
                                self.i = ((self._index) - (self.raw.backspaceCount));
                                while (self.i
                                    < ((self.raw.newCharCount)
                                        + ((self._index) - (self.raw.backspaceCount))))
                                {
                                    {
                                        self.raw.macroKey.push(self.TypingWord[(self.i) as usize]);
                                    }
                                    self.i += 1;
                                }
                            }
                        }
                    }
                    if (self.settings.vUpperCaseFirstChar != 0) {
                        if ((self._index == 1) && (self._upperCaseStatus == 2)) {
                            self.upperCaseFirstCharacter();
                        }
                        self._upperCaseStatus = ((0) as u8 as i64);
                    }
                    if ((self.IS_BRACKET_KEY(data) != 0)
                        && (((self
                            .IS_BRACKET_KEY(((self.raw.charData[(0) as usize]) as u16 as i64))
                            != 0)
                            || (self.settings.vInputType == vSimpleTelex1))
                            || (self.settings.vInputType == vSimpleTelex2)))
                    {
                        if (((self._index)
                            - (if (self.raw.code == vWillProcess) {
                                self.raw.backspaceCount
                            } else {
                                0
                            }))
                            > 0)
                        {
                            self._index = ((self._index - 1) as u8 as i64);
                            self.saveWord();
                        }
                        self._index = ((0) as u8 as i64);
                        self.tempDisableKey = 0;
                        self._stateIndex = ((0) as u8 as i64);
                        self.raw.extCode = ((3) as u8 as i64);
                        self._specialChar
                            .push(((data) | (if (self._isCaps != 0) { CAPS_MASK } else { 0 })));
                    }
                }
            }
        }
    }
}
