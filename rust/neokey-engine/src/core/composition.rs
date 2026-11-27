// Ported from Engine.cpp; preserve upstream scratch-counter and packed-output semantics.
use super::*;
impl Core {
    // Port of Engine.cpp::insertD.
    pub(super) fn insertD(&mut self, data: i64, isCaps: i64) {
        self.raw.code = ((vWillProcess) as u8 as i64);
        self.raw.backspaceCount = ((0) as u8 as i64);
        self.ii = ((self._index) - (1));
        while (self.ii >= 0) {
            {
                self.raw.backspaceCount = ((self.raw.backspaceCount + 1) as u8 as i64);
                if (self.chr(self.ii) == KEY_D) {
                    if (((self.TypingWord[(self.ii) as usize]) & (TONE_MASK)) != 0) {
                        self.raw.code = ((vRestore) as u8 as i64);
                        self.TypingWord[(self.ii) as usize] &= (!(TONE_MASK));
                        self.raw.charData[(((self._index) - (1)) - (self.ii)) as usize] =
                            self.TypingWord[(self.ii) as usize];
                        self.tempDisableKey = 1;
                        break;
                    } else {
                        self.TypingWord[(self.ii) as usize] |= TONE_MASK;
                        self.raw.charData[(((self._index) - (1)) - (self.ii)) as usize] =
                            self.getCharacterCode(self.TypingWord[(self.ii) as usize]);
                    }
                    break;
                } else {
                    self.raw.charData[(((self._index) - (1)) - (self.ii)) as usize] =
                        self.getCharacterCode(self.TypingWord[(self.ii) as usize]);
                }
            }
            self.ii -= 1;
        }
        self.raw.newCharCount = ((self.raw.backspaceCount) as u8 as i64);
    }
    // Port of Engine.cpp::insertAOE.
    pub(super) fn insertAOE(&mut self, data: i64, isCaps: i64) {
        self.findAndCalculateVowel(0);
        self.ii = self.vowelStartIndex;
        while (self.ii <= self.vowelEndIndex) {
            {
                self.TypingWord[(self.ii) as usize] &= (!(TONEW_MASK));
            }
            self.ii += 1;
        }
        self.raw.code = ((vWillProcess) as u8 as i64);
        self.raw.backspaceCount = ((0) as u8 as i64);
        self.ii = ((self._index) - (1));
        while (self.ii >= 0) {
            {
                self.raw.backspaceCount = ((self.raw.backspaceCount + 1) as u8 as i64);
                if (self.chr(self.ii) == data) {
                    if (((self.TypingWord[(self.ii) as usize]) & (TONE_MASK)) != 0) {
                        self.raw.code = ((vRestore) as u8 as i64);
                        self.TypingWord[(self.ii) as usize] &= (!(TONE_MASK));
                        self.raw.charData[(((self._index) - (1)) - (self.ii)) as usize] =
                            self.TypingWord[(self.ii) as usize];
                        if (data != KEY_O) {
                            self.tempDisableKey = 1;
                        }
                        break;
                    } else {
                        self.TypingWord[(self.ii) as usize] |= TONE_MASK;
                        if !(self.IS_KEY_D(data) != 0) {
                            self.TypingWord[(self.ii) as usize] &= (!(TONEW_MASK));
                        }
                        self.raw.charData[(((self._index) - (1)) - (self.ii)) as usize] =
                            self.getCharacterCode(self.TypingWord[(self.ii) as usize]);
                    }
                    break;
                } else {
                    self.raw.charData[(((self._index) - (1)) - (self.ii)) as usize] =
                        self.getCharacterCode(self.TypingWord[(self.ii) as usize]);
                }
            }
            self.ii -= 1;
        }
        self.raw.newCharCount = ((self.raw.backspaceCount) as u8 as i64);
    }
    // Port of Engine.cpp::insertW.
    pub(super) fn insertW(&mut self, data: i64, isCaps: i64) {
        self.isRestoredW = 0;
        self.findAndCalculateVowel(0);
        self.ii = self.vowelStartIndex;
        while (self.ii <= self.vowelEndIndex) {
            {
                self.TypingWord[(self.ii) as usize] &= (!(TONE_MASK));
            }
            self.ii += 1;
        }
        if (self.vowelCount > 1) {
            self.raw.backspaceCount = (((self._index) - (self.vowelStartIndex)) as u8 as i64);
            self.raw.newCharCount = ((self.raw.backspaceCount) as u8 as i64);
            if ((((((self.TypingWord[(self.vowelStartIndex) as usize]) & (TONEW_MASK)) != 0)
                && (((self.TypingWord[((self.vowelStartIndex) + (1)) as usize]) & (TONEW_MASK))
                    != 0))
                || ((((self.TypingWord[(self.vowelStartIndex) as usize]) & (TONEW_MASK)) != 0)
                    && (self.chr(((self.vowelStartIndex) + (1))) == KEY_I)))
                || ((((self.TypingWord[(self.vowelStartIndex) as usize]) & (TONEW_MASK)) != 0)
                    && (self.chr(((self.vowelStartIndex) + (1))) == KEY_A)))
            {
                self.raw.code = ((vRestore) as u8 as i64);
                self.ii = self.vowelStartIndex;
                while (self.ii < self._index) {
                    {
                        self.TypingWord[(self.ii) as usize] &= (!(TONEW_MASK));
                        self.raw.charData[(((self._index) - (1)) - (self.ii)) as usize] = ((self
                            .getCharacterCode(self.TypingWord[(self.ii) as usize]))
                            & (!(STANDALONE_MASK)));
                    }
                    self.ii += 1;
                }
                self.isRestoredW = 1;
                self.tempDisableKey = 1;
            } else {
                self.raw.code = ((vWillProcess) as u8 as i64);
                if ((self.chr(self.vowelStartIndex) == KEY_U)
                    && (self.chr(((self.vowelStartIndex) + (1))) == KEY_O))
                {
                    if (((((self.vowelStartIndex) - (2)) >= 0)
                        && (self.TypingWord[((self.vowelStartIndex) - (2)) as usize] == KEY_T))
                        && (self.TypingWord[((self.vowelStartIndex) - (1)) as usize] == KEY_H))
                    {
                        self.TypingWord[((self.vowelStartIndex) + (1)) as usize] |= TONEW_MASK;
                        if ((((self.vowelStartIndex) + (2)) < self._index)
                            && (self.chr(((self.vowelStartIndex) + (2))) == KEY_N))
                        {
                            self.TypingWord[(self.vowelStartIndex) as usize] |= TONEW_MASK;
                        }
                    } else {
                        if ((((self.vowelStartIndex) - (1)) >= 0)
                            && (self.TypingWord[((self.vowelStartIndex) - (1)) as usize] == KEY_Q))
                        {
                            self.TypingWord[((self.vowelStartIndex) + (1)) as usize] |= TONEW_MASK;
                        } else {
                            self.TypingWord[(self.vowelStartIndex) as usize] |= TONEW_MASK;
                            self.TypingWord[((self.vowelStartIndex) + (1)) as usize] |= TONEW_MASK;
                        }
                    }
                } else {
                    if (((((self.chr(self.vowelStartIndex) == KEY_U)
                        && (self.chr(((self.vowelStartIndex) + (1))) == KEY_A))
                        || ((self.chr(self.vowelStartIndex) == KEY_U)
                            && (self.chr(((self.vowelStartIndex) + (1))) == KEY_I)))
                        || ((self.chr(self.vowelStartIndex) == KEY_U)
                            && (self.chr(((self.vowelStartIndex) + (1))) == KEY_U)))
                        || ((self.chr(self.vowelStartIndex) == KEY_O)
                            && (self.chr(((self.vowelStartIndex) + (1))) == KEY_I)))
                    {
                        self.TypingWord[(self.vowelStartIndex) as usize] |= TONEW_MASK;
                    } else {
                        if (((self.chr(self.vowelStartIndex) == KEY_I)
                            && (self.chr(((self.vowelStartIndex) + (1))) == KEY_O))
                            || ((self.chr(self.vowelStartIndex) == KEY_O)
                                && (self.chr(((self.vowelStartIndex) + (1))) == KEY_A)))
                        {
                            self.TypingWord[((self.vowelStartIndex) + (1)) as usize] |= TONEW_MASK;
                        } else {
                            self.tempDisableKey = 1;
                            self.isChanged = 0;
                            self.raw.code = ((vDoNothing) as u8 as i64);
                        }
                    }
                }
                self.ii = self.vowelStartIndex;
                while (self.ii < self._index) {
                    {
                        self.raw.charData[(((self._index) - (1)) - (self.ii)) as usize] =
                            self.getCharacterCode(self.TypingWord[(self.ii) as usize]);
                    }
                    self.ii += 1;
                }
            }
            return;
        }
        self.raw.code = ((vWillProcess) as u8 as i64);
        self.raw.backspaceCount = ((0) as u8 as i64);
        self.ii = ((self._index) - (1));
        while (self.ii >= 0) {
            {
                if (self.ii < self.vowelStartIndex) {
                    break;
                }
                self.raw.backspaceCount = ((self.raw.backspaceCount + 1) as u8 as i64);
                match self.chr(self.ii) {
                    KEY_A | KEY_U | KEY_O => {
                        if (((self.TypingWord[(self.ii) as usize]) & (TONEW_MASK)) != 0) {
                            if (((self.TypingWord[(self.ii) as usize]) & (STANDALONE_MASK)) != 0) {
                                self.raw.code = ((vWillProcess) as u8 as i64);
                                if (self.chr(self.ii) == KEY_U) {
                                    self.TypingWord[(self.ii) as usize] = ((KEY_W)
                                        | (if (((self.TypingWord[(self.ii) as usize])
                                            & (CAPS_MASK))
                                            != 0)
                                        {
                                            CAPS_MASK
                                        } else {
                                            0
                                        }));
                                } else {
                                    if (self.chr(self.ii) == KEY_O) {
                                        self.raw.code = ((vRestore) as u8 as i64);
                                        self.TypingWord[(self.ii) as usize] = ((KEY_O)
                                            | (if (((self.TypingWord[(self.ii) as usize])
                                                & (CAPS_MASK))
                                                != 0)
                                            {
                                                CAPS_MASK
                                            } else {
                                                0
                                            }));
                                        self.isRestoredW = 1;
                                    }
                                }
                                self.raw.charData[(((self._index) - (1)) - (self.ii)) as usize] =
                                    self.TypingWord[(self.ii) as usize];
                            } else {
                                self.raw.code = ((vRestore) as u8 as i64);
                                self.TypingWord[(self.ii) as usize] &= (!(TONEW_MASK));
                                self.raw.charData[(((self._index) - (1)) - (self.ii)) as usize] =
                                    self.TypingWord[(self.ii) as usize];
                                self.isRestoredW = 1;
                            }
                            self.tempDisableKey = 1;
                        } else {
                            self.TypingWord[(self.ii) as usize] |= TONEW_MASK;
                            self.TypingWord[(self.ii) as usize] &= (!(TONE_MASK));
                            self.raw.charData[(((self._index) - (1)) - (self.ii)) as usize] =
                                self.getCharacterCode(self.TypingWord[(self.ii) as usize]);
                        }
                    }
                    _ => {
                        self.raw.charData[(((self._index) - (1)) - (self.ii)) as usize] =
                            self.getCharacterCode(self.TypingWord[(self.ii) as usize]);
                    }
                }
            }
            self.ii -= 1;
        }
        self.raw.newCharCount = ((self.raw.backspaceCount) as u8 as i64);
        if (self.isRestoredW != 0) {}
    }
    // Port of Engine.cpp::reverseLastStandaloneChar.
    pub(super) fn reverseLastStandaloneChar(&mut self, keyCode: i64, isCaps: i64) {
        self.raw.code = ((vWillProcess) as u8 as i64);
        self.raw.backspaceCount = ((0) as u8 as i64);
        self.raw.newCharCount = ((1) as u8 as i64);
        self.raw.extCode = ((4) as u8 as i64);
        self.TypingWord[((self._index) - (1)) as usize] = ((((keyCode) | (TONEW_MASK))
            | (STANDALONE_MASK))
            | (if (isCaps != 0) { CAPS_MASK } else { 0 }));
        self.raw.charData[(0) as usize] =
            self.getCharacterCode(self.TypingWord[((self._index) - (1)) as usize]);
    }
    // Port of Engine.cpp::checkForStandaloneChar.
    pub(super) fn checkForStandaloneChar(&mut self, data: i64, isCaps: i64, keyWillReverse: i64) {
        if ((self.chr(((self._index) - (1))) == keyWillReverse)
            && (((self.TypingWord[((self._index) - (1)) as usize]) & (TONEW_MASK)) != 0))
        {
            self.raw.code = ((vWillProcess) as u8 as i64);
            self.raw.backspaceCount = ((1) as u8 as i64);
            self.raw.newCharCount = ((1) as u8 as i64);
            self.TypingWord[((self._index) - (1)) as usize] =
                ((data) | (if (isCaps != 0) { CAPS_MASK } else { 0 }));
            self.raw.charData[(0) as usize] =
                self.getCharacterCode(self.TypingWord[((self._index) - (1)) as usize]);
            return;
        }
        if (((self._index > 0) && (self.chr(((self._index) - (1))) == KEY_U))
            && (keyWillReverse == KEY_O))
        {
            self.insertKey(keyWillReverse, isCaps, 1);
            self.reverseLastStandaloneChar(keyWillReverse, isCaps);
            return;
        }
        if (self._index == 0) {
            self.insertKey(data, isCaps, 0);
            self.reverseLastStandaloneChar(keyWillReverse, isCaps);
            return;
        } else {
            if (self._index == 1) {
                self.i = 0;
                while (self.i < (_standaloneWbad.len() as i64)) {
                    {
                        if (self.chr(0) == _standaloneWbad[(self.i) as usize]) {
                            self.insertKey(data, isCaps, 1);
                            return;
                        }
                    }
                    self.i += 1;
                }
                self.insertKey(data, isCaps, 0);
                self.reverseLastStandaloneChar(keyWillReverse, isCaps);
                return;
            } else {
                if (self._index == 2) {
                    self.i = 0;
                    while (self.i < (_doubleWAllowed.len() as i64)) {
                        {
                            if ((self.chr(0) == _doubleWAllowed[(self.i) as usize][(0) as usize])
                                && (self.chr(1)
                                    == _doubleWAllowed[(self.i) as usize][(1) as usize]))
                            {
                                self.insertKey(data, isCaps, 0);
                                self.reverseLastStandaloneChar(keyWillReverse, isCaps);
                                return;
                            }
                        }
                        self.i += 1;
                    }
                    self.insertKey(data, isCaps, 1);
                    return;
                }
            }
        }
        self.insertKey(data, isCaps, 1);
    }
    // Port of Engine.cpp::upperCaseFirstCharacter.
    pub(super) fn upperCaseFirstCharacter(&mut self) {
        if !(((self.TypingWord[(0) as usize]) & (CAPS_MASK)) != 0) {
            self.raw.code = ((vWillProcess) as u8 as i64);
            self.raw.backspaceCount = ((0) as u8 as i64);
            self.raw.newCharCount = ((1) as u8 as i64);
            self.TypingWord[(0) as usize] |= CAPS_MASK;
            self.raw.charData[(0) as usize] = self.getCharacterCode(self.TypingWord[(0) as usize]);
            self._upperCaseStatus = ((0) as u8 as i64);
            if (self.settings.vUseMacro != 0) {
                self.raw.macroKey[(0) as usize] |= CAPS_MASK;
            }
        }
    }
    // Port of Engine.cpp::handleMainKey.
    pub(super) fn handleMainKey(&mut self, data: i64, isCaps: i64) {
        if (self.IS_KEY_Z(data) != 0) {
            self.removeMark();
            if !(self.isChanged != 0) {
                self.insertKey(data, isCaps, 1);
            }
            return;
        }
        if (data == KEY_LEFT_BRACKET) {
            self.checkForStandaloneChar(data, isCaps, KEY_O);
            return;
        }
        if (data == KEY_RIGHT_BRACKET) {
            self.checkForStandaloneChar(data, isCaps, KEY_U);
            return;
        }
        if (self.IS_KEY_D(data) != 0) {
            self.isCorect = 0;
            self.isChanged = 0;
            self.k = self._index;
            self.i = 0;
            while (self.i < (_consonantD.len() as i64)) {
                {
                    if (self._index < (_consonantD[(self.i) as usize].len() as i64)) {
                        self.i += 1;
                        continue;
                    }
                    self.isCorect = 1;
                    self.checkCorrectVowel(&_consonantD, self.i, data);
                    if (((!(self.isCorect != 0) && (((self._index) - (2)) >= 0))
                        && (self.chr(((self._index) - (1))) == KEY_D))
                        && (self.IS_CONSONANT(self.chr(((self._index) - (2)))) != 0))
                    {
                        self.isCorect = 1;
                    }
                    if (self.isCorect != 0) {
                        self.isChanged = 1;
                        self.insertD(data, isCaps);
                        break;
                    }
                }
                self.i += 1;
            }
            if !(self.isChanged != 0) {
                self.insertKey(data, isCaps, 1);
            }
            return;
        }
        if (self.IS_MARK_KEY(data) != 0) {
            for vowelEntry in _vowelForMark.values() {
                let charset = vowelEntry.as_slice();
                self.isCorect = 0;
                self.isChanged = 0;
                self.k = self._index;
                self.l = 0;
                while (self.l < (charset.len() as i64)) {
                    {
                        if (self._index < (charset[(self.l) as usize].len() as i64)) {
                            self.l += 1;
                            continue;
                        }
                        self.isCorect = 1;
                        self.checkCorrectVowel(charset, self.l, data);
                        if (self.isCorect != 0) {
                            self.isChanged = 1;
                            if (self.IS_KEY_S(data) != 0) {
                                self.insertMark(MARK1_MASK, 1);
                            } else {
                                if (self.IS_KEY_F(data) != 0) {
                                    self.insertMark(MARK2_MASK, 1);
                                } else {
                                    if (self.IS_KEY_R(data) != 0) {
                                        self.insertMark(MARK3_MASK, 1);
                                    } else {
                                        if (self.IS_KEY_X(data) != 0) {
                                            self.insertMark(MARK4_MASK, 1);
                                        } else {
                                            if (self.IS_KEY_J(data) != 0) {
                                                self.insertMark(MARK5_MASK, 1);
                                            }
                                        }
                                    }
                                }
                            }
                            break;
                        }
                    }
                    self.l += 1;
                }
                if (self.isCorect != 0) {
                    break;
                }
            }
            if !(self.isChanged != 0) {
                self.insertKey(data, isCaps, 1);
            }
            return;
        }
        if (self.settings.vInputType == vVNI) {
            self.i = ((self._index) - (1));
            while (self.i >= 0) {
                {
                    if (((self.chr(self.i) == KEY_O) || (self.chr(self.i) == KEY_A))
                        || (self.chr(self.i) == KEY_E))
                    {
                        self.vowelEndIndex = ((self.i) as u8 as i64);
                        break;
                    }
                }
                self.i -= 1;
            }
        }
        self.keyForAEO = ((if (self.settings.vInputType != vVNI) {
            data
        } else {
            (if ((data == KEY_7) || (data == KEY_8)) {
                KEY_W
            } else {
                (if (data == KEY_6) {
                    self.TypingWord[(self.vowelEndIndex) as usize]
                } else {
                    data
                })
            })
        }) as u16 as i64);
        let charset = _vowel
            .get(&(self.keyForAEO))
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        self.isCorect = 0;
        self.isChanged = 0;
        self.k = self._index;
        self.i = 0;
        while (self.i < (charset.len() as i64)) {
            {
                if (self._index < (charset[(self.i) as usize].len() as i64)) {
                    self.i += 1;
                    continue;
                }
                self.isCorect = 1;
                self.checkCorrectVowel(charset, self.i, data);
                if (self.isCorect != 0) {
                    self.isChanged = 1;
                    if (self.IS_KEY_DOUBLE(data) != 0) {
                        self.insertAOE(self.keyForAEO, isCaps);
                    } else {
                        if (self.IS_KEY_W(data) != 0) {
                            if (self.settings.vInputType == vVNI) {
                                self.j = ((self._index) - (1));
                                while (self.j >= 0) {
                                    {
                                        if ((((self.chr(self.j) == KEY_O)
                                            || (self.chr(self.j) == KEY_U))
                                            || (self.chr(self.j) == KEY_A))
                                            || (self.chr(self.j) == KEY_E))
                                        {
                                            self.vowelEndIndex = ((self.j) as u8 as i64);
                                            break;
                                        }
                                    }
                                    self.j -= 1;
                                }
                                if ((((data == KEY_7) && (self.chr(self.vowelEndIndex) == KEY_A))
                                    && ((if (((self.vowelEndIndex) - (1)) >= 0) {
                                        i64::from(
                                            (self.chr(((self.vowelEndIndex) - (1)))) != (KEY_U),
                                        )
                                    } else {
                                        1
                                    }) != 0))
                                    || ((data == KEY_8)
                                        && ((self.chr(self.vowelEndIndex) == KEY_O)
                                            || (self.chr(self.vowelEndIndex) == KEY_U))))
                                {
                                    break;
                                }
                            }
                            self.insertW(self.keyForAEO, isCaps);
                        }
                    }
                    break;
                }
            }
            self.i += 1;
        }
        if !(self.isChanged != 0) {
            if ((data == KEY_W) && (self.settings.vInputType != vSimpleTelex1)) {
                self.checkForStandaloneChar(data, isCaps, KEY_U);
            } else {
                self.insertKey(data, isCaps, 1);
            }
        }
    }
    // Port of Engine.cpp::handleQuickTelex.
    pub(super) fn handleQuickTelex(&mut self, data: i64, isCaps: i64) {
        self.raw.code = ((vWillProcess) as u8 as i64);
        self.raw.backspaceCount = ((1) as u8 as i64);
        self.raw.newCharCount = ((2) as u8 as i64);
        self.raw.charData[(1) as usize] = ((_quickTelex.get(&(data)).cloned().unwrap_or_default()
            [(0) as usize])
            | (if (isCaps != 0) { CAPS_MASK } else { 0 }));
        self.raw.charData[(0) as usize] = ((_quickTelex.get(&(data)).cloned().unwrap_or_default()
            [(1) as usize])
            | (if (isCaps != 0) { CAPS_MASK } else { 0 }));
        self.insertKey(
            _quickTelex.get(&(data)).cloned().unwrap_or_default()[(1) as usize],
            isCaps,
            0,
        );
    }
    // Port of Engine.cpp::checkQuickConsonant.
    pub(super) fn checkQuickConsonant(&mut self) -> i64 {
        if (self._index <= 1) {
            return 0;
        }
        self.l = 0;
        if (self._index > 0) {
            if ((self.settings.vQuickStartConsonant != 0)
                && (i64::from(_quickStartConsonant.contains_key(&(self.chr(0)))) != 0))
            {
                self.raw.code = ((vRestore) as u8 as i64);
                self.raw.backspaceCount = ((self._index) as u8 as i64);
                self.raw.newCharCount = (((self._index) + (1)) as u8 as i64);
                if (self._index < ((MAX_BUFF) - (1))) {
                    self._index = ((self._index + 1) as u8 as i64);
                }
                self.i = ((self._index) - (1));
                while (self.i >= 2) {
                    {
                        self.TypingWord[(self.i) as usize] =
                            self.TypingWord[((self.i) - (1)) as usize];
                    }
                    self.i -= 1;
                }
                self.TypingWord[(1) as usize] = ((_quickStartConsonant
                    .get(&(self.chr(0)))
                    .cloned()
                    .unwrap_or_default()[(1) as usize])
                    | (if ((((self.TypingWord[(0) as usize]) & (CAPS_MASK)) != 0)
                        && (((self.TypingWord[(2) as usize]) & (CAPS_MASK)) != 0))
                    {
                        CAPS_MASK
                    } else {
                        0
                    }));
                self.TypingWord[(0) as usize] = ((_quickStartConsonant
                    .get(&(self.chr(0)))
                    .cloned()
                    .unwrap_or_default()[(0) as usize])
                    | (if (((self.TypingWord[(0) as usize]) & (CAPS_MASK)) != 0) {
                        CAPS_MASK
                    } else {
                        0
                    }));
                self.l = 1;
            }
            if (((self.settings.vQuickEndConsonant != 0)
                && ((((self._index) - (2)) >= 0)
                    && !(self.IS_CONSONANT(self.chr(((self._index) - (2)))) != 0)))
                && (i64::from(_quickEndConsonant.contains_key(&(self.chr(((self._index) - (1))))))
                    != 0))
            {
                self.raw.code = ((vRestore) as u8 as i64);
                if (self.l == 1) {
                    self.raw.newCharCount = ((self.raw.newCharCount + 1) as u8 as i64);
                } else {
                    self.raw.backspaceCount = ((1) as u8 as i64);
                    self.raw.newCharCount = ((2) as u8 as i64);
                }
                if (self._index < ((MAX_BUFF) - (1))) {
                    self._index = ((self._index + 1) as u8 as i64);
                }
                self.TypingWord[((self._index) - (1)) as usize] = ((_quickEndConsonant
                    .get(&(self.chr(((self._index) - (2)))))
                    .cloned()
                    .unwrap_or_default()[(1) as usize])
                    | (if (((self.TypingWord[((self._index) - (2)) as usize]) & (CAPS_MASK)) != 0) {
                        CAPS_MASK
                    } else {
                        0
                    }));
                self.TypingWord[((self._index) - (2)) as usize] = ((_quickEndConsonant
                    .get(&(self.chr(((self._index) - (2)))))
                    .cloned()
                    .unwrap_or_default()[(0) as usize])
                    | (if (((self.TypingWord[((self._index) - (2)) as usize]) & (CAPS_MASK)) != 0) {
                        CAPS_MASK
                    } else {
                        0
                    }));
                self.l = 1;
            }
            if (self.l == 1) {
                self._hasHandleQuickConsonant = 1;
                self.i = ((self._index) - (1));
                while (self.i >= 0) {
                    {
                        self.raw.charData[(((self._index) - (1)) - (self.i)) as usize] =
                            self.getCharacterCode(self.TypingWord[(self.i) as usize]);
                    }
                    self.i -= 1;
                }
                return 1;
            }
        }
        return 0;
    }
}
