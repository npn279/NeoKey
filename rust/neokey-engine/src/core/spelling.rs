// Ported from Engine.cpp; preserve upstream scratch-counter and packed-output semantics.
use super::*;
// All the consonant keys fit within the low 64 macOS virtual key codes.
const START_BY_KEY: [[&[i64]; 4]; 64] = consonants_by_first_key(START_CONSONANTS);
const END_BY_KEY: [[&[i64]; 4]; 64] = consonants_by_first_key(END_CONSONANTS);

const fn consonants_by_first_key(rows: &[&'static [i64]]) -> [[&'static [i64]; 4]; 64] {
    let mut groups = [[&[] as &[i64]; 4]; 64];
    let mut counts = [0; 64];
    let mut row = 0;
    while row < rows.len() {
        let key = (rows[row][0] & !(CONSONANT_ALLOW_MASK | END_CONSONANT_MASK)) as usize;
        groups[key][counts[key]] = rows[row];
        counts[key] += 1;
        row += 1;
    }
    groups
}

fn consonants_for(groups: &'static [[&[i64]; 4]; 64], first: i64) -> &'static [&'static [i64]] {
    let key = first & !(CONSONANT_ALLOW_MASK | END_CONSONANT_MASK);
    groups.get(key as usize).map_or(&[], |group| group)
}

impl Core {
    // Port of Engine.cpp::checkSpelling.
    pub(super) fn checkSpelling(&mut self, forceCheckVowel: i64) {
        let word = &self.TypingWord[..self._index as usize];
        let spelling = match word.split_last() {
            Some((last, prefix)) if last & CHAR_MASK == KEY_RIGHT_BRACKET => prefix,
            _ => word,
        };
        let Some(&first) = spelling.first() else {
            self.tempDisableKey = 0;
            return;
        };
        let first = first & CHAR_MASK;
        let mut start = 0;
        if self.IS_CONSONANT(first) != 0 {
            let quick_mask = if self.settings.vQuickStartConsonant != 0 {
                END_CONSONANT_MASK
            } else {
                0
            };
            let allow_mask = if self.settings.vAllowConsonantZFWJ != 0 {
                CONSONANT_ALLOW_MASK
            } else {
                0
            };
            // The first complete match wins. Preserve the original row order within each group.
            for row in consonants_for(&START_BY_KEY, first) {
                let Some((&initial, tail)) = row.split_first() else {
                    break;
                };
                if (initial & !quick_mask == first || initial & !allow_mask == first)
                    && spelling.get(1..row.len()).is_some_and(|keys| {
                        tail.iter()
                            .zip(keys)
                            .all(|(&expected, &key)| expected == key & CHAR_MASK)
                    })
                {
                    start = row.len();
                    break;
                }
            }
        }
        let mut spelling_ok = start == spelling.len();
        if start > 0
            && start + 1 < spelling.len()
            && spelling[start] & CHAR_MASK == KEY_U
            && spelling[start - 1] & CHAR_MASK == KEY_Q
        {
            start += 1;
        } else if word.len() >= 2
            && first == KEY_G
            && word[1] & CHAR_MASK == KEY_I
            // The original looks at slot 2 even when only two keys are active.
            && self.IS_CONSONANT(self.TypingWord[2] & CHAR_MASK) != 0
        {
            start = 1;
        }
        self.vowelStartIndex = start as i64;
        let (vowels, ending) = spelling[start..].split_at(
            spelling[start..]
                .iter()
                .take(3)
                .take_while(|&&key| self.IS_CONSONANT(key & CHAR_MASK) == 0)
                .count(),
        );
        if !vowels.is_empty() {
            self.vowelEndIndex = (start + vowels.len()) as i64;
            if vowels.len() > 1 && forceCheckVowel != 0 {
                let remaining = &spelling[start..];
                let vowel_ok = _vowelCombine
                    .get(&(remaining[0] & CHAR_MASK))
                    .is_some_and(|rows| {
                        rows.iter().any(|row| {
                            let (&allows_ending, pattern) =
                                row.split_first().expect("vowel pattern");
                            (ending.is_empty() || allows_ending != 0)
                                && pattern.iter().zip(remaining).all(|(&expected, &key)| {
                                    expected == key & (CHAR_MASK | TONE_MASK | TONEW_MASK)
                                })
                                && remaining
                                    .get(pattern.len())
                                    .is_none_or(|key| self.IS_CONSONANT(key & CHAR_MASK) != 0)
                        })
                    });
                if !vowel_ok {
                    self.tempDisableKey = 1;
                    return;
                }
            }
            spelling_ok |= match ending {
                [] => true,
                [first, ..] if ending.len() <= 2 => {
                    let first = first & CHAR_MASK;
                    let quick_mask = if self.settings.vQuickEndConsonant != 0 {
                        END_CONSONANT_MASK
                    } else {
                        0
                    };
                    // A partially typed ending is valid if it is a prefix of any row.
                    consonants_for(&END_BY_KEY, first).iter().any(|row| {
                        row.len() >= ending.len()
                            && row
                                .iter()
                                .zip(ending)
                                .all(|(&expected, &key)| expected & !quick_mask == key & CHAR_MASK)
                    })
                }
                _ => false,
            };
            if spelling_ok {
                let before_coda = match word {
                    [.., vowel, c, h] if c & CHAR_MASK == KEY_C && h & CHAR_MASK == KEY_H => *vowel,
                    [.., vowel, t] if t & CHAR_MASK == KEY_T => *vowel,
                    _ => 0,
                };
                spelling_ok =
                    before_coda & MARK_MASK == 0 || before_coda & (MARK1_MASK | MARK5_MASK) != 0;
            }
        }
        self.tempDisableKey = i64::from(!spelling_ok);
    }
    // Port of Engine.cpp::checkGrammar.
    pub(super) fn checkGrammar(&mut self, deltaBackSpace: i64) {
        if ((self._index <= 1) || (self._index >= MAX_BUFF)) {
            return;
        }
        self.findAndCalculateVowel(1);
        if (self.vowelCount == 0) {
            return;
        }
        self.isCheckedGrammar = 0;
        self.l = self.vowelStartIndex;
        if (self._index >= 3) {
            self.i = ((self._index) - (1));
            while (self.i >= 0) {
                {
                    if ((((((self.chr(self.i) == KEY_N) || (self.chr(self.i) == KEY_C))
                        || (self.chr(self.i) == KEY_I))
                        || (self.chr(self.i) == KEY_M))
                        || (self.chr(self.i) == KEY_P))
                        || (self.chr(self.i) == KEY_T))
                    {
                        if (((((self.i) - (2)) >= 0) && (self.chr(((self.i) - (1))) == KEY_O))
                            && (self.chr(((self.i) - (2))) == KEY_U))
                        {
                            if ((((self.TypingWord[((self.i) - (1)) as usize]) & (TONEW_MASK))
                                ^ ((self.TypingWord[((self.i) - (2)) as usize]) & (TONEW_MASK)))
                                != 0)
                            {
                                self.TypingWord[((self.i) - (2)) as usize] |= TONEW_MASK;
                                self.TypingWord[((self.i) - (1)) as usize] |= TONEW_MASK;
                                self.isCheckedGrammar = 1;
                                break;
                            }
                        }
                    }
                }
                self.i -= 1;
            }
        }
        if (self._index >= 2) {
            self.i = self.l;
            while (self.i <= self.vowelEndIndex) {
                {
                    if (((self.TypingWord[(self.i) as usize]) & (MARK_MASK)) != 0) {
                        let mut mark: i64 = ((self.TypingWord[(self.i) as usize]) & (MARK_MASK));
                        self.TypingWord[(self.i) as usize] &= (!(MARK_MASK));
                        self.insertMark(mark, 0);
                        if (self.i != self.vowelWillSetMark) {
                            self.isCheckedGrammar = 1;
                        }
                        break;
                    }
                }
                self.i += 1;
            }
        }
        if (self.isCheckedGrammar != 0) {
            if (self.raw.code == vDoNothing) {
                self.raw.code = ((vWillProcess) as u8 as i64);
            }
            self.raw.backspaceCount = ((0) as u8 as i64);
            self.i = ((self._index) - (1));
            while (self.i >= self.l) {
                {
                    self.raw.backspaceCount = ((self.raw.backspaceCount + 1) as u8 as i64);
                    self.raw.charData[(((self._index) - (1)) - (self.i)) as usize] =
                        self.getCharacterCode(self.TypingWord[(self.i) as usize]);
                }
                self.i -= 1;
            }
            self.raw.newCharCount = ((self.raw.backspaceCount) as u8 as i64);
            self.raw.backspaceCount = (((self.raw.backspaceCount) + (deltaBackSpace)) as u8 as i64);
            self.raw.extCode = ((4) as u8 as i64);
        }
    }
    // Port of Engine.cpp::checkCorrectVowel.
    pub(super) fn checkCorrectVowel(&mut self, charset: &[Vec<i64>], row: i64, markKey: i64) {
        if (((self._index >= 2) && (self.chr(((self._index) - (1))) == KEY_U))
            && (self.chr(((self._index) - (2))) == KEY_Q))
        {
            self.isCorect = 0;
            return;
        }
        self.k = ((self._index) - (1));
        self.j = (((charset[(row) as usize].len() as i64) as i32 as i64) - (1));
        while (self.j >= 0) {
            {
                if (((charset[(row) as usize][(self.j) as usize])
                    & (!(if (self.settings.vQuickEndConsonant != 0) {
                        END_CONSONANT_MASK
                    } else {
                        0
                    })))
                    != self.chr(self.k))
                {
                    self.isCorect = 0;
                    return;
                }
                self.k -= 1;
                if (self.k < 0) {
                    break;
                }
            }
            self.j -= 1;
        }
        if (((self.isCorect != 0) && ((charset[(row) as usize].len() as i64) > 1))
            && (((self.IS_KEY_F(markKey) != 0) || (self.IS_KEY_X(markKey) != 0))
                || (self.IS_KEY_R(markKey) != 0)))
        {
            if ((charset[(row) as usize][(1) as usize] == KEY_C)
                || (charset[(row) as usize][(1) as usize] == KEY_T))
            {
                self.isCorect = 0;
            } else {
                if (((charset[(row) as usize].len() as i64) > 2)
                    && (charset[(row) as usize][(2) as usize] == KEY_T))
                {
                    self.isCorect = 0;
                }
            }
        }
        if ((self.isCorect != 0) && (self.k >= 0)) {
            if (self.chr(self.k) == self.chr(((self.k) + (1)))) {
                self.isCorect = 0;
            }
        }
    }
}
