// Ported from Engine.cpp; preserve upstream scratch-counter and packed-output semantics.
use super::*;
impl Core {
    // Port of Engine.cpp::getCharacterCode.
    pub(crate) fn getCharacterCode(&mut self, data: i64) -> i64 {
        let caps = if data & CAPS_MASK != 0 { 0 } else { 1 };
        let mut key = data & CHAR_MASK;
        let mark = data & MARK_MASK;
        let values = if mark != 0 {
            let mut index = match mark {
                MARK1_MASK => 0,
                MARK2_MASK => 2,
                MARK3_MASK => 4,
                MARK4_MASK => 6,
                MARK5_MASK => 8,
                _ => -2,
            } + caps;
            if [KEY_A, KEY_O, KEY_U, KEY_E].contains(&key) && data & (TONE_MASK | TONEW_MASK) == 0 {
                index += 4;
            }
            if data & TONE_MASK != 0 {
                key |= TONE_MASK;
            } else if data & TONEW_MASK != 0 {
                key |= TONEW_MASK;
            }
            _codeTable[self.settings.vCodeTable as usize]
                .get(&key)
                .and_then(|v| v.get(index as usize))
                .copied()
        } else {
            let index = if data & TONE_MASK != 0 {
                caps
            } else if data & TONEW_MASK != 0 {
                caps + 2
            } else {
                return data;
            };
            _codeTable[self.settings.vCodeTable as usize]
                .get(&key)
                .and_then(|v| v.get(index as usize))
                .copied()
        };
        values.map(|v| v | CHAR_CODE_MASK).unwrap_or(data)
    }
    // Port of Engine.cpp::findAndCalculateVowel.
    pub(super) fn findAndCalculateVowel(&mut self, forGrammar: i64) {
        self.vowelCount = ((0) as u8 as i64);
        self.vowelStartIndex = (({
            let value = ((0) as u8 as i64);
            let target = &mut self.vowelEndIndex;
            *target = value;
            target.clone()
        }) as u8 as i64);
        self.iii = ((self._index) - (1));
        while (self.iii >= 0) {
            {
                if (self.IS_CONSONANT(self.chr(self.iii)) != 0) {
                    if (self.vowelCount > 0) {
                        break;
                    }
                } else {
                    if (self.vowelCount == 0) {
                        self.vowelEndIndex = ((self.iii) as u8 as i64);
                    }
                    if !(forGrammar != 0) {
                        if (((((self.iii) - (1)) >= 0)
                            && ((self.chr(self.iii) == KEY_I)
                                && (self.chr(((self.iii) - (1))) == KEY_G)))
                            || ((((self.iii) - (1)) >= 0)
                                && ((self.chr(self.iii) == KEY_U)
                                    && (self.chr(((self.iii) - (1))) == KEY_Q))))
                        {
                            break;
                        }
                    }
                    self.vowelStartIndex = ((self.iii) as u8 as i64);
                    self.vowelCount = ((self.vowelCount + 1) as u8 as i64);
                }
            }
            self.iii -= 1;
        }
        if (((((self.vowelStartIndex) - (1)) >= 0) && (self.chr(self.vowelStartIndex) == KEY_U))
            && (self.chr(((self.vowelStartIndex) - (1))) == KEY_Q))
        {
            self.vowelStartIndex = ((self.vowelStartIndex + 1) as u8 as i64);
            self.vowelCount = ((self.vowelCount - 1) as u8 as i64);
        }
    }
    // Port of Engine.cpp::removeMark.
    pub(super) fn removeMark(&mut self) {
        self.findAndCalculateVowel(1);
        self.isChanged = 0;
        if (self._index > 0) {
            self.i = self.vowelStartIndex;
            while (self.i <= self.vowelEndIndex) {
                {
                    if (((self.TypingWord[(self.i) as usize]) & (MARK_MASK)) != 0) {
                        self.TypingWord[(self.i) as usize] &= (!(MARK_MASK));
                        self.isChanged = 1;
                    }
                }
                self.i += 1;
            }
        }
        if (self.isChanged != 0) {
            self.raw.code = ((vWillProcess) as u8 as i64);
            self.raw.backspaceCount = ((0) as u8 as i64);
            self.i = ((self._index) - (1));
            while (self.i >= self.vowelStartIndex) {
                {
                    self.raw.backspaceCount = ((self.raw.backspaceCount + 1) as u8 as i64);
                    self.raw.charData[(((self._index) - (1)) - (self.i)) as usize] =
                        self.getCharacterCode(self.TypingWord[(self.i) as usize]);
                }
                self.i -= 1;
            }
            self.raw.newCharCount = ((self.raw.backspaceCount) as u8 as i64);
        } else {
            self.raw.code = ((vDoNothing) as u8 as i64);
        }
    }
    // Port of Engine.cpp::canHasEndConsonant.
    pub(super) fn canHasEndConsonant(&mut self) -> i64 {
        let vo = _vowelCombine
            .get(&(self.chr(self.vowelStartIndex)))
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        self.ii = 0;
        while (self.ii < (vo.len() as i64)) {
            {
                self.kk = self.vowelStartIndex;
                self.iii = 1;
                while (self.iii < (vo[(self.ii) as usize].len() as i64)) {
                    {
                        if ((self.kk > self.vowelEndIndex)
                            || ((((self.chr(self.kk))
                                | ((self.TypingWord[(self.kk) as usize]) & (TONE_MASK)))
                                | ((self.TypingWord[(self.kk) as usize]) & (TONEW_MASK)))
                                != vo[(self.ii) as usize][(self.iii) as usize]))
                        {
                            break;
                        }
                        self.kk += 1;
                    }
                    self.iii += 1;
                }
                if (self.iii >= (vo[(self.ii) as usize].len() as i64)) {
                    return i64::from((vo[(self.ii) as usize][(0) as usize]) == (1));
                }
            }
            self.ii += 1;
        }
        return 0;
    }
    // Port of Engine.cpp::handleModernMark.
    pub(super) fn handleModernMark(&mut self) {
        self.vowelWillSetMark = ((self.vowelEndIndex) as u8 as i64);
        self.raw.backspaceCount = (((self._index) - (self.vowelEndIndex)) as u8 as i64);
        if ((self.vowelCount == 3)
            && ((((((self.chr(self.vowelStartIndex) == KEY_O)
                && (self.chr(((self.vowelStartIndex) + (1))) == KEY_A))
                && (self.chr(((self.vowelStartIndex) + (2))) == KEY_I))
                || (((self.chr(self.vowelStartIndex) == KEY_U)
                    && (self.chr(((self.vowelStartIndex) + (1))) == KEY_Y))
                    && (self.chr(((self.vowelStartIndex) + (2))) == KEY_U)))
                || (((self.chr(self.vowelStartIndex) == KEY_O)
                    && (self.chr(((self.vowelStartIndex) + (1))) == KEY_E))
                    && (self.chr(((self.vowelStartIndex) + (2))) == KEY_O)))
                || (((self.chr(self.vowelStartIndex) == KEY_U)
                    && (self.chr(((self.vowelStartIndex) + (1))) == KEY_Y))
                    && (self.chr(((self.vowelStartIndex) + (2))) == KEY_A))))
        {
            self.vowelWillSetMark = (((self.vowelStartIndex) + (1)) as u8 as i64);
            self.raw.backspaceCount = (((self._index) - (self.vowelWillSetMark)) as u8 as i64);
        } else {
            if ((((self.chr(self.vowelStartIndex) == KEY_O)
                && (self.chr(((self.vowelStartIndex) + (1))) == KEY_I))
                || ((self.chr(self.vowelStartIndex) == KEY_A)
                    && (self.chr(((self.vowelStartIndex) + (1))) == KEY_I)))
                || ((self.chr(self.vowelStartIndex) == KEY_U)
                    && (self.chr(((self.vowelStartIndex) + (1))) == KEY_I)))
            {
                self.vowelWillSetMark = ((self.vowelStartIndex) as u8 as i64);
                self.raw.backspaceCount = (((self._index) - (self.vowelWillSetMark)) as u8 as i64);
            } else {
                if ((self.chr(((self.vowelEndIndex) - (1))) == KEY_A)
                    && (self.chr(self.vowelEndIndex) == KEY_Y))
                {
                    self.vowelWillSetMark = (((self.vowelEndIndex) - (1)) as u8 as i64);
                    self.raw.backspaceCount =
                        ((((self._index) - (self.vowelEndIndex)) + (1)) as u8 as i64);
                } else {
                    if ((self.chr(self.vowelStartIndex) == KEY_U)
                        && (self.chr(((self.vowelStartIndex) + (1))) == KEY_O))
                    {
                        self.vowelWillSetMark = (((self.vowelStartIndex) + (1)) as u8 as i64);
                        self.raw.backspaceCount =
                            (((self._index) - (self.vowelWillSetMark)) as u8 as i64);
                    } else {
                        if ((self.chr(((self.vowelStartIndex) + (1))) == KEY_O)
                            || (self.chr(((self.vowelStartIndex) + (1))) == KEY_U))
                        {
                            self.vowelWillSetMark = (((self.vowelEndIndex) - (1)) as u8 as i64);
                            self.raw.backspaceCount =
                                ((((self._index) - (self.vowelEndIndex)) + (1)) as u8 as i64);
                        } else {
                            if ((self.chr(self.vowelStartIndex) == KEY_O)
                                || (self.chr(self.vowelStartIndex) == KEY_U))
                            {
                                self.vowelWillSetMark = ((self.vowelEndIndex) as u8 as i64);
                                self.raw.backspaceCount =
                                    (((self._index) - (self.vowelEndIndex)) as u8 as i64);
                            }
                        }
                    }
                }
            }
        }
        if (((((self.chr(self.vowelStartIndex) == KEY_I)
            && (((self.TypingWord[((self.vowelStartIndex) + (1)) as usize])
                & ((KEY_E) | (TONE_MASK)))
                != 0))
            || ((self.chr(self.vowelStartIndex) == KEY_Y)
                && (((self.TypingWord[((self.vowelStartIndex) + (1)) as usize])
                    & ((KEY_E) | (TONE_MASK)))
                    != 0)))
            || ((self.chr(self.vowelStartIndex) == KEY_U)
                && (self.TypingWord[((self.vowelStartIndex) + (1)) as usize]
                    == ((KEY_O) | (TONE_MASK)))))
            || ((self.TypingWord[(self.vowelStartIndex) as usize] == ((KEY_U) | (TONEW_MASK)))
                && (self.TypingWord[((self.vowelStartIndex) + (1)) as usize]
                    == ((KEY_O) | (TONEW_MASK)))))
        {
            if (((self.vowelStartIndex) + (2)) < self._index) {
                if (((((((((((self.chr(((self.vowelStartIndex) + (2))) == KEY_P)
                    || (self.chr(((self.vowelStartIndex) + (2))) == KEY_T))
                    || (self.chr(((self.vowelStartIndex) + (2))) == KEY_M))
                    || (self.chr(((self.vowelStartIndex) + (2))) == KEY_N))
                    || (self.chr(((self.vowelStartIndex) + (2))) == KEY_O))
                    || (self.chr(((self.vowelStartIndex) + (2))) == KEY_U))
                    || (self.chr(((self.vowelStartIndex) + (2))) == KEY_I))
                    || (self.chr(((self.vowelStartIndex) + (2))) == KEY_C))
                    || (((((self.vowelStartIndex) + (3)) < self._index)
                        && (self.chr(((self.vowelStartIndex) + (2))) == KEY_C))
                        && (self.chr(((self.vowelStartIndex) + (2))) == KEY_H)))
                    || (((((self.vowelStartIndex) + (3)) < self._index)
                        && (self.chr(((self.vowelStartIndex) + (2))) == KEY_N))
                        && (self.chr(((self.vowelStartIndex) + (2))) == KEY_H)))
                    || (((((self.vowelStartIndex) + (3)) < self._index)
                        && (self.chr(((self.vowelStartIndex) + (2))) == KEY_N))
                        && (self.chr(((self.vowelStartIndex) + (2))) == KEY_G)))
                {
                    self.vowelWillSetMark = (((self.vowelStartIndex) + (1)) as u8 as i64);
                    self.raw.backspaceCount =
                        (((self._index) - (self.vowelWillSetMark)) as u8 as i64);
                } else {
                    self.vowelWillSetMark = ((self.vowelStartIndex) as u8 as i64);
                    self.raw.backspaceCount =
                        (((self._index) - (self.vowelWillSetMark)) as u8 as i64);
                }
            } else {
                self.vowelWillSetMark = ((self.vowelStartIndex) as u8 as i64);
                self.raw.backspaceCount = (((self._index) - (self.vowelWillSetMark)) as u8 as i64);
            }
        } else {
            if (((((self.chr(self.vowelStartIndex) == KEY_I)
                && (self.chr(self.vowelStartIndex) == KEY_A))
                || ((self.chr(self.vowelStartIndex) == KEY_Y)
                    && (self.chr(self.vowelStartIndex) == KEY_A)))
                || ((self.chr(self.vowelStartIndex) == KEY_U)
                    && (self.chr(self.vowelStartIndex) == KEY_A)))
                || ((self.chr(self.vowelStartIndex) == KEY_U)
                    && (self.TypingWord[((self.vowelStartIndex) + (1)) as usize]
                        == ((KEY_U) | (TONEW_MASK)))))
            {
                self.vowelWillSetMark = ((self.vowelStartIndex) as u8 as i64);
                self.raw.backspaceCount = (((self._index) - (self.vowelWillSetMark)) as u8 as i64);
            }
        }
        if (self.vowelCount == 2) {
            if ((((self.chr(self.vowelStartIndex) == KEY_I)
                && (self.chr(((self.vowelStartIndex) + (1))) == KEY_A))
                || ((self.chr(self.vowelStartIndex) == KEY_I)
                    && (self.chr(((self.vowelStartIndex) + (1))) == KEY_U)))
                || ((self.chr(self.vowelStartIndex) == KEY_I)
                    && (self.chr(((self.vowelStartIndex) + (1))) == KEY_O)))
            {
                if ((self.vowelStartIndex == 0)
                    || (self.chr(((self.vowelStartIndex) - (1))) != KEY_G))
                {
                    self.vowelWillSetMark = ((self.vowelStartIndex) as u8 as i64);
                    self.raw.backspaceCount =
                        (((self._index) - (self.vowelWillSetMark)) as u8 as i64);
                } else {
                    self.vowelWillSetMark = (((self.vowelStartIndex) + (1)) as u8 as i64);
                    self.raw.backspaceCount =
                        (((self._index) - (self.vowelWillSetMark)) as u8 as i64);
                }
            } else {
                if ((self.chr(self.vowelStartIndex) == KEY_U)
                    && (self.chr(((self.vowelStartIndex) + (1))) == KEY_A))
                {
                    if ((self.vowelStartIndex == 0)
                        || (self.chr(((self.vowelStartIndex) - (1))) != KEY_Q))
                    {
                        if ((((self.vowelEndIndex) + (1)) >= self._index)
                            || !(self.canHasEndConsonant() != 0))
                        {
                            self.vowelWillSetMark = ((self.vowelStartIndex) as u8 as i64);
                            self.raw.backspaceCount =
                                (((self._index) - (self.vowelWillSetMark)) as u8 as i64);
                        }
                    } else {
                        self.vowelWillSetMark = (((self.vowelStartIndex) + (1)) as u8 as i64);
                        self.raw.backspaceCount =
                            (((self._index) - (self.vowelWillSetMark)) as u8 as i64);
                    }
                } else {
                    if ((self.chr(self.vowelStartIndex) == KEY_O)
                        && (self.chr(((self.vowelStartIndex) + (1))) == KEY_O))
                    {
                        self.vowelWillSetMark = ((self.vowelEndIndex) as u8 as i64);
                        self.raw.backspaceCount =
                            (((self._index) - (self.vowelWillSetMark)) as u8 as i64);
                    }
                }
            }
        }
    }
    // Port of Engine.cpp::handleOldMark.
    pub(super) fn handleOldMark(&mut self) {
        if ((self.vowelCount == 0) && (self.chr(self.vowelEndIndex) == KEY_I)) {
            self.vowelWillSetMark = ((self.vowelEndIndex) as u8 as i64);
        } else {
            self.vowelWillSetMark = ((self.vowelStartIndex) as u8 as i64);
        }
        self.raw.backspaceCount = (((self._index) - (self.vowelWillSetMark)) as u8 as i64);
        if ((self.vowelCount == 3)
            || (((((self.vowelEndIndex) + (1)) < self._index)
                && (self.IS_CONSONANT(self.chr(((self.vowelEndIndex) + (1)))) != 0))
                && (self.canHasEndConsonant() != 0)))
        {
            self.vowelWillSetMark = (((self.vowelStartIndex) + (1)) as u8 as i64);
            self.raw.backspaceCount = (((self._index) - (self.vowelWillSetMark)) as u8 as i64);
        }
        self.ii = self.vowelStartIndex;
        while (self.ii <= self.vowelEndIndex) {
            {
                if (((self.chr(self.ii) == KEY_E)
                    && (((self.TypingWord[(self.ii) as usize]) & (TONE_MASK)) != 0))
                    || ((self.chr(self.ii) == KEY_O)
                        && (((self.TypingWord[(self.ii) as usize]) & (TONEW_MASK)) != 0)))
                {
                    self.vowelWillSetMark = ((self.ii) as u8 as i64);
                    self.raw.backspaceCount =
                        (((self._index) - (self.vowelWillSetMark)) as u8 as i64);
                    break;
                }
            }
            self.ii += 1;
        }
        self.raw.newCharCount = ((self.raw.backspaceCount) as u8 as i64);
    }
    // Port of Engine.cpp::insertMark.
    pub(super) fn insertMark(&mut self, markMask: i64, canModifyFlag: i64) {
        self.vowelCount = ((0) as u8 as i64);
        if (canModifyFlag != 0) {
            self.raw.code = ((vWillProcess) as u8 as i64);
        }
        self.raw.backspaceCount = (({
            let value = ((0) as u8 as i64);
            let target = &mut self.raw.newCharCount;
            *target = value;
            target.clone()
        }) as u8 as i64);
        self.findAndCalculateVowel(0);
        self.vowelWillSetMark = ((0) as u8 as i64);
        if (self.vowelCount == 1) {
            self.vowelWillSetMark = ((self.vowelEndIndex) as u8 as i64);
            self.raw.backspaceCount = (((self._index) - (self.vowelEndIndex)) as u8 as i64);
        } else {
            if (self.settings.vUseModernOrthography == 0) {
                self.handleOldMark();
            } else {
                self.handleModernMark();
            }
            if ((((self.TypingWord[(self.vowelEndIndex) as usize]) & (TONE_MASK)) != 0)
                || (((self.TypingWord[(self.vowelEndIndex) as usize]) & (TONEW_MASK)) != 0))
            {
                self.vowelWillSetMark = ((self.vowelEndIndex) as u8 as i64);
            }
        }
        self.kk = (((self._index) - (1)) - (self.vowelStartIndex));
        if (((self.TypingWord[(self.vowelWillSetMark) as usize]) & (markMask)) != 0) {
            self.TypingWord[(self.vowelWillSetMark) as usize] &= (!(MARK_MASK));
            if (canModifyFlag != 0) {
                self.raw.code = ((vRestore) as u8 as i64);
            }
            self.ii = self.vowelStartIndex;
            while (self.ii < self._index) {
                {
                    self.TypingWord[(self.ii) as usize] &= (!(MARK_MASK));
                    self.raw.charData[({
                        let old = self.kk;
                        self.kk -= 1;
                        old
                    }) as usize] = self.getCharacterCode(self.TypingWord[(self.ii) as usize]);
                }
                self.ii += 1;
            }
            self.tempDisableKey = 1;
        } else {
            self.TypingWord[(self.vowelWillSetMark) as usize] &= (!(MARK_MASK));
            self.TypingWord[(self.vowelWillSetMark) as usize] |= markMask;
            self.ii = self.vowelStartIndex;
            while (self.ii < self._index) {
                {
                    if (self.ii != self.vowelWillSetMark) {
                        self.TypingWord[(self.ii) as usize] &= (!(MARK_MASK));
                    }
                    self.raw.charData[({
                        let old = self.kk;
                        self.kk -= 1;
                        old
                    }) as usize] = self.getCharacterCode(self.TypingWord[(self.ii) as usize]);
                }
                self.ii += 1;
            }
            self.raw.backspaceCount = (((self._index) - (self.vowelStartIndex)) as u8 as i64);
        }
        self.raw.newCharCount = ((self.raw.backspaceCount) as u8 as i64);
    }
}
