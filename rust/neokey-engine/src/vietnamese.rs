#![allow(non_upper_case_globals)]
// Exact OpenKey Vietnamese tables. Keys use macOS virtual key codes.
use crate::data::*;
use std::{collections::BTreeMap, sync::LazyLock};
pub static douKey: LazyLock<Vec<Vec<i64>>> =
    LazyLock::new(|| vec![vec![KEY_A, 0xE2], vec![KEY_E, 0xEA]]);
pub static _vowel: LazyLock<BTreeMap<i64, Vec<Vec<i64>>>> = LazyLock::new(|| {
    BTreeMap::from([
        (
            KEY_A,
            vec![
                vec![KEY_A, KEY_N, KEY_G],
                vec![KEY_A, KEY_G | END_CONSONANT_MASK],
                vec![KEY_A, KEY_N],
                vec![KEY_A, KEY_M],
                vec![KEY_A, KEY_U],
                vec![KEY_A, KEY_Y],
                vec![KEY_A, KEY_T],
                vec![KEY_A, KEY_P],
                vec![KEY_A],
                vec![KEY_A, KEY_C],
            ],
        ),
        (
            KEY_O,
            vec![
                vec![KEY_O, KEY_N, KEY_G],
                vec![KEY_O, KEY_G | END_CONSONANT_MASK],
                vec![KEY_O, KEY_N],
                vec![KEY_O, KEY_M],
                vec![KEY_O, KEY_I],
                vec![KEY_O, KEY_C],
                vec![KEY_O, KEY_T],
                vec![KEY_O, KEY_P],
                vec![KEY_O],
            ],
        ),
        (
            KEY_E,
            vec![
                vec![KEY_E, KEY_N, KEY_H],
                vec![KEY_E, KEY_H | END_CONSONANT_MASK],
                vec![KEY_E, KEY_N, KEY_G],
                vec![KEY_E, KEY_G | END_CONSONANT_MASK],
                vec![KEY_E, KEY_C, KEY_H],
                vec![KEY_E, KEY_K | END_CONSONANT_MASK],
                vec![KEY_E, KEY_C],
                vec![KEY_E, KEY_T],
                vec![KEY_E, KEY_Y],
                vec![KEY_E, KEY_U],
                vec![KEY_E, KEY_P],
                vec![KEY_E, KEY_C],
                vec![KEY_E, KEY_N],
                vec![KEY_E, KEY_M],
                vec![KEY_E],
            ],
        ),
        (
            KEY_W,
            vec![
                vec![KEY_O, KEY_N],
                vec![KEY_U, KEY_O, KEY_N, KEY_G],
                vec![KEY_U, KEY_O, KEY_G | END_CONSONANT_MASK],
                vec![KEY_U, KEY_O, KEY_N],
                vec![KEY_U, KEY_O, KEY_I],
                vec![KEY_U, KEY_O, KEY_C],
                vec![KEY_O, KEY_I],
                vec![KEY_O, KEY_P],
                vec![KEY_O, KEY_M],
                vec![KEY_O, KEY_A],
                vec![KEY_O, KEY_T],
                vec![KEY_U, KEY_N, KEY_G],
                vec![KEY_U, KEY_G | END_CONSONANT_MASK],
                vec![KEY_A, KEY_N, KEY_G],
                vec![KEY_A, KEY_G | END_CONSONANT_MASK],
                vec![KEY_U, KEY_N],
                vec![KEY_U, KEY_M],
                vec![KEY_U, KEY_C],
                vec![KEY_U, KEY_A],
                vec![KEY_U, KEY_I],
                vec![KEY_U, KEY_T],
                vec![KEY_U],
                vec![KEY_A, KEY_P],
                vec![KEY_A, KEY_T],
                vec![KEY_A, KEY_M],
                vec![KEY_A, KEY_N],
                vec![KEY_A],
                vec![KEY_A, KEY_C],
                vec![KEY_A, KEY_C, KEY_H],
                vec![KEY_A, KEY_K | END_CONSONANT_MASK],
                vec![KEY_O],
                vec![KEY_U, KEY_U],
            ],
        ),
    ])
});
pub static _vowelCombine: LazyLock<BTreeMap<i64, Vec<Vec<i64>>>> = LazyLock::new(|| {
    BTreeMap::from([
        (
            KEY_A,
            vec![
                vec![0, KEY_A, KEY_I],
                vec![0, KEY_A, KEY_O],
                vec![0, KEY_A, KEY_U],
                vec![0, KEY_A | TONE_MASK, KEY_U],
                vec![0, KEY_A, KEY_Y],
                vec![0, KEY_A | TONE_MASK, KEY_Y],
            ],
        ),
        (
            KEY_E,
            vec![vec![0, KEY_E, KEY_O], vec![0, KEY_E | TONE_MASK, KEY_U]],
        ),
        (
            KEY_I,
            vec![
                vec![1, KEY_I, KEY_E | TONE_MASK, KEY_U],
                vec![0, KEY_I, KEY_A],
                vec![1, KEY_I, KEY_E | TONE_MASK],
                vec![0, KEY_I, KEY_U],
            ],
        ),
        (
            KEY_O,
            vec![
                vec![0, KEY_O, KEY_A, KEY_I],
                vec![0, KEY_O, KEY_A, KEY_O],
                vec![0, KEY_O, KEY_A, KEY_Y],
                vec![0, KEY_O, KEY_E, KEY_O],
                vec![1, KEY_O, KEY_A],
                vec![1, KEY_O, KEY_A | TONEW_MASK],
                vec![1, KEY_O, KEY_E],
                vec![0, KEY_O, KEY_I],
                vec![0, KEY_O | TONE_MASK, KEY_I],
                vec![0, KEY_O | TONEW_MASK, KEY_I],
                vec![1, KEY_O, KEY_O],
                vec![1, KEY_O | TONE_MASK, KEY_O | TONE_MASK],
            ],
        ),
        (
            KEY_U,
            vec![
                vec![0, KEY_U, KEY_Y, KEY_U],
                vec![1, KEY_U, KEY_Y, KEY_E | TONE_MASK],
                vec![0, KEY_U, KEY_Y, KEY_A],
                vec![0, KEY_U | TONEW_MASK, KEY_O | TONEW_MASK, KEY_U],
                vec![0, KEY_U | TONEW_MASK, KEY_O | TONEW_MASK, KEY_I],
                vec![0, KEY_U, KEY_O | TONE_MASK, KEY_I],
                vec![0, KEY_U, KEY_A | TONE_MASK, KEY_Y],
                vec![1, KEY_U, KEY_A, KEY_O],
                vec![1, KEY_U, KEY_A],
                vec![1, KEY_U, KEY_A | TONEW_MASK],
                vec![1, KEY_U, KEY_A | TONE_MASK],
                vec![0, KEY_U | TONEW_MASK, KEY_A],
                vec![1, KEY_U, KEY_E | TONE_MASK],
                vec![0, KEY_U, KEY_I],
                vec![0, KEY_U | TONEW_MASK, KEY_I],
                vec![1, KEY_U, KEY_O],
                vec![1, KEY_U, KEY_O | TONE_MASK],
                vec![0, KEY_U, KEY_O | TONEW_MASK],
                vec![1, KEY_U | TONEW_MASK, KEY_O | TONEW_MASK],
                vec![0, KEY_U | TONEW_MASK, KEY_U],
                vec![1, KEY_U, KEY_Y],
            ],
        ),
        (
            KEY_Y,
            vec![
                vec![0, KEY_Y, KEY_E | TONE_MASK, KEY_U],
                vec![1, KEY_Y, KEY_E | TONE_MASK],
            ],
        ),
    ])
});
pub static _consonantD: LazyLock<Vec<Vec<i64>>> = LazyLock::new(|| {
    vec![
        vec![KEY_D, KEY_E, KEY_N, KEY_H],
        vec![KEY_D, KEY_E, KEY_H | END_CONSONANT_MASK],
        vec![KEY_D, KEY_E, KEY_N, KEY_G],
        vec![KEY_D, KEY_E, KEY_G | END_CONSONANT_MASK],
        vec![KEY_D, KEY_E, KEY_C, KEY_H],
        vec![KEY_D, KEY_E, KEY_K | END_CONSONANT_MASK],
        vec![KEY_D, KEY_E, KEY_N],
        vec![KEY_D, KEY_E, KEY_C],
        vec![KEY_D, KEY_E, KEY_M],
        vec![KEY_D, KEY_E],
        vec![KEY_D, KEY_E, KEY_T],
        vec![KEY_D, KEY_E, KEY_U],
        vec![KEY_D, KEY_E, KEY_O],
        vec![KEY_D, KEY_E, KEY_P],
        vec![KEY_D, KEY_U, KEY_N, KEY_G],
        vec![KEY_D, KEY_U, KEY_G | END_CONSONANT_MASK],
        vec![KEY_D, KEY_U, KEY_N],
        vec![KEY_D, KEY_U, KEY_M],
        vec![KEY_D, KEY_U, KEY_C],
        vec![KEY_D, KEY_U, KEY_O],
        vec![KEY_D, KEY_U, KEY_A],
        vec![KEY_D, KEY_U, KEY_O, KEY_I],
        vec![KEY_D, KEY_U, KEY_O, KEY_C],
        vec![KEY_D, KEY_U, KEY_O, KEY_N],
        vec![KEY_D, KEY_U, KEY_O, KEY_N, KEY_G],
        vec![KEY_D, KEY_U, KEY_O, KEY_G | END_CONSONANT_MASK],
        vec![KEY_D, KEY_U],
        vec![KEY_D, KEY_U, KEY_P],
        vec![KEY_D, KEY_U, KEY_T],
        vec![KEY_D, KEY_U, KEY_I],
        vec![KEY_D, KEY_I, KEY_C, KEY_H],
        vec![KEY_D, KEY_I, KEY_K | END_CONSONANT_MASK],
        vec![KEY_D, KEY_I, KEY_C],
        vec![KEY_D, KEY_I, KEY_N, KEY_H],
        vec![KEY_D, KEY_I, KEY_H | END_CONSONANT_MASK],
        vec![KEY_D, KEY_I, KEY_N],
        vec![KEY_D, KEY_I],
        vec![KEY_D, KEY_I, KEY_A],
        vec![KEY_D, KEY_I, KEY_E],
        vec![KEY_D, KEY_I, KEY_E, KEY_C],
        vec![KEY_D, KEY_I, KEY_E, KEY_U],
        vec![KEY_D, KEY_I, KEY_E, KEY_N],
        vec![KEY_D, KEY_I, KEY_E, KEY_M],
        vec![KEY_D, KEY_I, KEY_E, KEY_P],
        vec![KEY_D, KEY_I, KEY_T],
        vec![KEY_D, KEY_O],
        vec![KEY_D, KEY_O, KEY_A],
        vec![KEY_D, KEY_O, KEY_A, KEY_N],
        vec![KEY_D, KEY_O, KEY_A, KEY_N, KEY_G],
        vec![KEY_D, KEY_O, KEY_A, KEY_G | END_CONSONANT_MASK],
        vec![KEY_D, KEY_O, KEY_A, KEY_N, KEY_H],
        vec![KEY_D, KEY_O, KEY_A, KEY_H | END_CONSONANT_MASK],
        vec![KEY_D, KEY_O, KEY_A, KEY_M],
        vec![KEY_D, KEY_O, KEY_E],
        vec![KEY_D, KEY_O, KEY_I],
        vec![KEY_D, KEY_O, KEY_P],
        vec![KEY_D, KEY_O, KEY_C],
        vec![KEY_D, KEY_O, KEY_N],
        vec![KEY_D, KEY_O, KEY_N, KEY_G],
        vec![KEY_D, KEY_O, KEY_G | END_CONSONANT_MASK],
        vec![KEY_D, KEY_O, KEY_M],
        vec![KEY_D, KEY_O, KEY_T],
        vec![KEY_D, KEY_A],
        vec![KEY_D, KEY_A, KEY_T],
        vec![KEY_D, KEY_A, KEY_Y],
        vec![KEY_D, KEY_A, KEY_U],
        vec![KEY_D, KEY_A, KEY_I],
        vec![KEY_D, KEY_A, KEY_O],
        vec![KEY_D, KEY_A, KEY_P],
        vec![KEY_D, KEY_A, KEY_C],
        vec![KEY_D, KEY_A, KEY_C, KEY_H],
        vec![KEY_D, KEY_A, KEY_K | END_CONSONANT_MASK],
        vec![KEY_D, KEY_A, KEY_N],
        vec![KEY_D, KEY_A, KEY_N, KEY_H],
        vec![KEY_D, KEY_A, KEY_H | END_CONSONANT_MASK],
        vec![KEY_D, KEY_A, KEY_N, KEY_G],
        vec![KEY_D, KEY_A, KEY_G | END_CONSONANT_MASK],
        vec![KEY_D, KEY_A, KEY_M],
        vec![KEY_D],
    ]
});
pub static _vowelForMark: LazyLock<BTreeMap<i64, Vec<Vec<i64>>>> = LazyLock::new(|| {
    BTreeMap::from([
        (
            KEY_A,
            vec![
                vec![KEY_A, KEY_N, KEY_G],
                vec![KEY_A, KEY_G | END_CONSONANT_MASK],
                vec![KEY_A, KEY_N],
                vec![KEY_A, KEY_N, KEY_H],
                vec![KEY_A, KEY_H | END_CONSONANT_MASK],
                vec![KEY_A, KEY_M],
                vec![KEY_A, KEY_U],
                vec![KEY_A, KEY_Y],
                vec![KEY_A, KEY_T],
                vec![KEY_A, KEY_P],
                vec![KEY_A],
                vec![KEY_A, KEY_C],
                vec![KEY_A, KEY_I],
                vec![KEY_A, KEY_O],
                vec![KEY_A, KEY_C, KEY_H],
                vec![KEY_A, KEY_K | END_CONSONANT_MASK],
            ],
        ),
        (
            KEY_O,
            vec![
                vec![KEY_O, KEY_O, KEY_N, KEY_G],
                vec![KEY_O, KEY_O, KEY_G | END_CONSONANT_MASK],
                vec![KEY_O, KEY_N, KEY_G],
                vec![KEY_O, KEY_G | END_CONSONANT_MASK],
                vec![KEY_O, KEY_O, KEY_N],
                vec![KEY_O, KEY_O, KEY_C],
                vec![KEY_O, KEY_O],
                vec![KEY_O, KEY_N],
                vec![KEY_O, KEY_M],
                vec![KEY_O, KEY_I],
                vec![KEY_O, KEY_C],
                vec![KEY_O, KEY_T],
                vec![KEY_O, KEY_P],
                vec![KEY_O],
            ],
        ),
        (
            KEY_E,
            vec![
                vec![KEY_E, KEY_N, KEY_H],
                vec![KEY_E, KEY_H | END_CONSONANT_MASK],
                vec![KEY_E, KEY_N, KEY_G],
                vec![KEY_E, KEY_G | END_CONSONANT_MASK],
                vec![KEY_E, KEY_C, KEY_H],
                vec![KEY_E, KEY_K | END_CONSONANT_MASK],
                vec![KEY_E, KEY_C],
                vec![KEY_E, KEY_T],
                vec![KEY_E, KEY_Y],
                vec![KEY_E, KEY_U],
                vec![KEY_E, KEY_P],
                vec![KEY_E, KEY_C],
                vec![KEY_E, KEY_N],
                vec![KEY_E, KEY_M],
                vec![KEY_E],
            ],
        ),
        (
            KEY_I,
            vec![
                vec![KEY_I, KEY_N, KEY_H],
                vec![KEY_I, KEY_H | END_CONSONANT_MASK],
                vec![KEY_I, KEY_C, KEY_H],
                vec![KEY_I, KEY_K | END_CONSONANT_MASK],
                vec![KEY_I, KEY_N],
                vec![KEY_I, KEY_T],
                vec![KEY_I, KEY_U],
                vec![KEY_I, KEY_U, KEY_P],
                vec![KEY_I, KEY_N],
                vec![KEY_I, KEY_M],
                vec![KEY_I, KEY_P],
                vec![KEY_I, KEY_A],
                vec![KEY_I, KEY_C],
                vec![KEY_I],
            ],
        ),
        (
            KEY_U,
            vec![
                vec![KEY_U, KEY_N, KEY_G],
                vec![KEY_U, KEY_G | END_CONSONANT_MASK],
                vec![KEY_U, KEY_I],
                vec![KEY_U, KEY_O],
                vec![KEY_U, KEY_Y],
                vec![KEY_U, KEY_Y, KEY_N],
                vec![KEY_U, KEY_Y, KEY_T],
                vec![KEY_U, KEY_Y, KEY_P],
                vec![KEY_U, KEY_Y, KEY_C, KEY_H],
                vec![KEY_U, KEY_Y, KEY_K | END_CONSONANT_MASK],
                vec![KEY_U, KEY_Y, KEY_N, KEY_H],
                vec![KEY_U, KEY_Y, KEY_H | END_CONSONANT_MASK],
                vec![KEY_U, KEY_T],
                vec![KEY_U, KEY_U],
                vec![KEY_U, KEY_A],
                vec![KEY_U, KEY_I],
                vec![KEY_U, KEY_C],
                vec![KEY_U, KEY_N],
                vec![KEY_U, KEY_M],
                vec![KEY_U, KEY_P],
                vec![KEY_U],
            ],
        ),
        (KEY_Y, vec![vec![KEY_Y]]),
    ])
});
pub(crate) const START_CONSONANTS: &[&[i64]] = &[
    &[KEY_N, KEY_G, KEY_H],
    &[KEY_P, KEY_H],
    &[KEY_T, KEY_H],
    &[KEY_T, KEY_R],
    &[KEY_G, KEY_I],
    &[KEY_C, KEY_H],
    &[KEY_N, KEY_H],
    &[KEY_N, KEY_G],
    &[KEY_K, KEY_H],
    &[KEY_G, KEY_H],
    &[KEY_G],
    &[KEY_C],
    &[KEY_Q],
    &[KEY_K],
    &[KEY_T],
    &[KEY_R],
    &[KEY_H],
    &[KEY_B],
    &[KEY_M],
    &[KEY_V],
    &[KEY_N],
    &[KEY_L],
    &[KEY_X],
    &[KEY_P],
    &[KEY_S],
    &[KEY_D],
    &[KEY_F | CONSONANT_ALLOW_MASK],
    &[KEY_W | CONSONANT_ALLOW_MASK],
    &[KEY_Z | CONSONANT_ALLOW_MASK],
    &[KEY_J | CONSONANT_ALLOW_MASK],
    &[KEY_F | END_CONSONANT_MASK],
    &[KEY_W | END_CONSONANT_MASK],
    &[KEY_J | END_CONSONANT_MASK],
];
pub static _consonantTable: LazyLock<Vec<Vec<i64>>> =
    LazyLock::new(|| START_CONSONANTS.iter().map(|row| row.to_vec()).collect());
pub(crate) const END_CONSONANTS: &[&[i64]] = &[
    &[KEY_T],
    &[KEY_P],
    &[KEY_C],
    &[KEY_N],
    &[KEY_M],
    &[KEY_G | END_CONSONANT_MASK],
    &[KEY_K | END_CONSONANT_MASK],
    &[KEY_H | END_CONSONANT_MASK],
    &[KEY_C, KEY_H],
    &[KEY_N, KEY_H],
    &[KEY_N, KEY_G],
];
pub static _endConsonantTable: LazyLock<Vec<Vec<i64>>> =
    LazyLock::new(|| END_CONSONANTS.iter().map(|row| row.to_vec()).collect());
pub static _standaloneWbad: LazyLock<Vec<i64>> =
    LazyLock::new(|| vec![KEY_W, KEY_E, KEY_Y, KEY_F, KEY_J, KEY_K, KEY_Z]);
pub static _doubleWAllowed: LazyLock<Vec<Vec<i64>>> = LazyLock::new(|| {
    vec![
        vec![KEY_T, KEY_R],
        vec![KEY_T, KEY_H],
        vec![KEY_C, KEY_H],
        vec![KEY_N, KEY_H],
        vec![KEY_N, KEY_G],
        vec![KEY_K, KEY_H],
        vec![KEY_G, KEY_I],
        vec![KEY_P, KEY_H],
        vec![KEY_G, KEY_H],
    ]
});
pub static _quickStartConsonant: LazyLock<BTreeMap<i64, Vec<i64>>> = LazyLock::new(|| {
    BTreeMap::from([
        (KEY_F, vec![KEY_P, KEY_H]),
        (KEY_J, vec![KEY_G, KEY_I]),
        (KEY_W, vec![KEY_Q, KEY_U]),
    ])
});
pub static _quickEndConsonant: LazyLock<BTreeMap<i64, Vec<i64>>> = LazyLock::new(|| {
    BTreeMap::from([
        (KEY_G, vec![KEY_N, KEY_G]),
        (KEY_H, vec![KEY_N, KEY_H]),
        (KEY_K, vec![KEY_C, KEY_H]),
    ])
});
pub static _codeTable: LazyLock<Vec<BTreeMap<i64, Vec<i64>>>> = LazyLock::new(|| {
    vec![
        BTreeMap::from([
            (
                KEY_A,
                vec![
                    0x00C2, 0x00E2, 0x0102, 0x0103, 0x00C1, 0x00E1, 0x00C0, 0x00E0, 0x1EA2, 0x1EA3,
                    0x00C3, 0x00E3, 0x1EA0, 0x1EA1,
                ],
            ),
            (
                KEY_O,
                vec![
                    0x00D4, 0x00F4, 0x01A0, 0x01A1, 0x00D3, 0x00F3, 0x00D2, 0x00F2, 0x1ECE, 0x1ECF,
                    0x00D5, 0x00F5, 0x1ECC, 0x1ECD,
                ],
            ),
            (
                KEY_U,
                vec![
                    0x0000, 0x0000, 0x01AF, 0x01B0, 0x00DA, 0x00FA, 0x00D9, 0x00F9, 0x1EE6, 0x1EE7,
                    0x0168, 0x0169, 0x1EE4, 0x1EE5,
                ],
            ),
            (
                KEY_E,
                vec![
                    0x00CA, 0x00EA, 0x0000, 0x0000, 0x00C9, 0x00E9, 0x00C8, 0x00E8, 0x1EBA, 0x1EBB,
                    0x1EBC, 0x1EBD, 0x1EB8, 0x1EB9,
                ],
            ),
            (KEY_D, vec![0x0110, 0x0111]),
            (
                KEY_A | TONE_MASK,
                vec![
                    0x1EA4, 0x1EA5, 0x1EA6, 0x1EA7, 0x1EA8, 0x1EA9, 0x1EAA, 0x1EAB, 0x1EAC, 0x1EAD,
                ],
            ),
            (
                KEY_A | TONEW_MASK,
                vec![
                    0x1EAE, 0x1EAF, 0x1EB0, 0x1EB1, 0x1EB2, 0x1EB3, 0x1EB4, 0x1EB5, 0x1EB6, 0x1EB7,
                ],
            ),
            (
                KEY_O | TONE_MASK,
                vec![
                    0x1ED0, 0x1ED1, 0x1ED2, 0x1ED3, 0x1ED4, 0x1ED5, 0x1ED6, 0x1ED7, 0x1ED8, 0x1ED9,
                ],
            ),
            (
                KEY_O | TONEW_MASK,
                vec![
                    0x1EDA, 0x1EDB, 0x1EDC, 0x1EDD, 0x1EDE, 0x1EDF, 0x1EE0, 0x1EE1, 0x1EE2, 0x1EE3,
                ],
            ),
            (
                KEY_U | TONEW_MASK,
                vec![
                    0x1EE8, 0x1EE9, 0x1EEA, 0x1EEB, 0x1EEC, 0x1EED, 0x1EEE, 0x1EEF, 0x1EF0, 0x1EF1,
                ],
            ),
            (
                KEY_E | TONE_MASK,
                vec![
                    0x1EBE, 0x1EBF, 0x1EC0, 0x1EC1, 0x1EC2, 0x1EC3, 0x1EC4, 0x1EC5, 0x1EC6, 0x1EC7,
                ],
            ),
            (
                KEY_I,
                vec![
                    0x00CD, 0x00ED, 0x00CC, 0x00EC, 0x1EC8, 0x1EC9, 0x128, 0x129, 0x1ECA, 0x1ECB,
                ],
            ),
            (
                KEY_Y,
                vec![
                    0x00DD, 0x00FD, 0x1EF2, 0x1EF3, 0x1EF6, 0x1EF7, 0x1EF8, 0x1EF9, 0x1EF4, 0x1EF5,
                ],
            ),
        ]),
        BTreeMap::from([
            (
                KEY_A,
                vec![
                    0xA2, 0xA9, 0xA1, 0xA8, 0xB8, 0xB8, 0xB5, 0xB5, 0xB6, 0xB6, 0xB7, 0xB7, 0xB9,
                    0xB9,
                ],
            ),
            (
                KEY_O,
                vec![
                    0xA4, 0xAB, 0xA5, 0xAC, 0xE3, 0xE3, 0xDF, 0xDF, 0xE1, 0xE1, 0xE2, 0xE2, 0xE4,
                    0xE4,
                ],
            ),
            (
                KEY_U,
                vec![
                    0x00, 0x00, 0xA6, 0xAD, 0xF3, 0xF3, 0xEF, 0xEF, 0xF1, 0xF1, 0xF2, 0xF2, 0xF4,
                    0xF4,
                ],
            ),
            (
                KEY_E,
                vec![
                    0xA3, 0xAA, 0x00, 0x00, 0xD0, 0xD0, 0xCC, 0xCC, 0xCE, 0xCE, 0xCF, 0xCF, 0xD1,
                    0xD1,
                ],
            ),
            (KEY_D, vec![0xA7, 0xAE]),
            (
                KEY_A | TONE_MASK,
                vec![0xCA, 0xCA, 0xC7, 0xC7, 0xC8, 0xC8, 0xC9, 0xC9, 0xCB, 0xCB],
            ),
            (
                KEY_A | TONEW_MASK,
                vec![0xBE, 0xBE, 0xBB, 0xBB, 0xBC, 0xBC, 0xBD, 0xBD, 0xC6, 0xC6],
            ),
            (
                KEY_O | TONE_MASK,
                vec![0xE8, 0xE8, 0xE5, 0xE5, 0xE6, 0xE6, 0xE7, 0xE7, 0xE9, 0xE9],
            ),
            (
                KEY_O | TONEW_MASK,
                vec![0xED, 0xED, 0xEA, 0xEA, 0xEB, 0xEB, 0xEC, 0xEC, 0xEE, 0xEE],
            ),
            (
                KEY_U | TONEW_MASK,
                vec![0xF8, 0xF8, 0xF5, 0xF5, 0xF6, 0xF6, 0xF7, 0xF7, 0xF9, 0xF9],
            ),
            (
                KEY_E | TONE_MASK,
                vec![0xD5, 0xD5, 0xD2, 0xD2, 0xD3, 0xD3, 0xD4, 0xD4, 0xD6, 0xD6],
            ),
            (
                KEY_I,
                vec![0xDD, 0xDD, 0xD7, 0xD7, 0xD8, 0xD8, 0xDc, 0xDc, 0xDe, 0xDe],
            ),
            (
                KEY_Y,
                vec![0xFD, 0xFD, 0xFA, 0xFA, 0xFB, 0xFB, 0xFC, 0xFC, 0xFE, 0xFE],
            ),
        ]),
        BTreeMap::from([
            (
                KEY_A,
                vec![
                    0xC241, 0xE261, 0xCA41, 0xEA61, 0xD941, 0xF961, 0xD841, 0xF861, 0xDB41, 0xFB61,
                    0xD541, 0xF561, 0xCF41, 0xEF61,
                ],
            ),
            (
                KEY_O,
                vec![
                    0xC24F, 0xE26F, 0x00D4, 0x00F4, 0xD94F, 0xF96F, 0xD84F, 0xF86F, 0xDB4F, 0xFB6F,
                    0xD54F, 0xF56F, 0xCF4F, 0xEF6F,
                ],
            ),
            (
                KEY_U,
                vec![
                    0x0000, 0x0000, 0x00D6, 0x00F6, 0xD955, 0xF975, 0xD855, 0xF875, 0xDB55, 0xFB75,
                    0xD555, 0xF575, 0xCF55, 0xEF75,
                ],
            ),
            (
                KEY_E,
                vec![
                    0xC245, 0xE265, 0x0000, 0x0000, 0xD945, 0xF965, 0xD845, 0xF865, 0xDB45, 0xFB65,
                    0xD545, 0xF565, 0xCF45, 0xEF65,
                ],
            ),
            (KEY_D, vec![0x00D1, 0x00F1]),
            (
                KEY_A | TONE_MASK,
                vec![
                    0xC141, 0xE161, 0xC041, 0xE061, 0xC541, 0xE561, 0xC341, 0xE361, 0xC441, 0xE461,
                ],
            ),
            (
                KEY_A | TONEW_MASK,
                vec![
                    0xC941, 0xE961, 0xC841, 0xE861, 0xDA41, 0xFA61, 0xDC41, 0xFC61, 0xCB41, 0xEB61,
                ],
            ),
            (
                KEY_O | TONE_MASK,
                vec![
                    0xC14F, 0xE16F, 0xC04F, 0xE06F, 0xC54F, 0xE56F, 0xC34F, 0xE36F, 0xC44F, 0xE46F,
                ],
            ),
            (
                KEY_O | TONEW_MASK,
                vec![
                    0xD9D4, 0xF9F4, 0xD8D4, 0xF8F4, 0xDBD4, 0xFBF4, 0xD5D4, 0xF5F4, 0xCFD4, 0xEFF4,
                ],
            ),
            (
                KEY_U | TONEW_MASK,
                vec![
                    0xD9D6, 0xF9F6, 0xD8D6, 0xF8F6, 0xDBD6, 0xFBF6, 0xD5D6, 0xF5F6, 0xCFD6, 0xEFF6,
                ],
            ),
            (
                KEY_E | TONE_MASK,
                vec![
                    0xC145, 0xE165, 0xC045, 0xE065, 0xC545, 0xE565, 0xC345, 0xE365, 0xC445, 0xE465,
                ],
            ),
            (
                KEY_I,
                vec![
                    0x00CD, 0x00ED, 0x00CC, 0x00EC, 0x00C6, 0x00E6, 0x00D3, 0x00F3, 0x00D2, 0x00F2,
                ],
            ),
            (
                KEY_Y,
                vec![
                    0xD959, 0xF979, 0xD859, 0xF879, 0xDB59, 0xFB79, 0xD559, 0xF579, 0x00CE, 0x00EE,
                ],
            ),
        ]),
        BTreeMap::from([
            (
                KEY_A,
                vec![
                    0x00C2, 0x00E2, 0x0102, 0x0103, 0x2041, 0x2061, 0x4041, 0x4061, 0x6041, 0x6061,
                    0x8041, 0x8061, 0xA041, 0xA061,
                ],
            ),
            (
                KEY_O,
                vec![
                    0x00D4, 0x00F4, 0x01A0, 0x01A1, 0x204F, 0x206F, 0x404F, 0x406F, 0x604F, 0x606F,
                    0x804F, 0x806F, 0xA04F, 0xA06F,
                ],
            ),
            (
                KEY_U,
                vec![
                    0x0000, 0x0000, 0x01AF, 0x01B0, 0x2055, 0x2075, 0x4055, 0x4075, 0x6055, 0x6075,
                    0x8055, 0x8075, 0xA055, 0xA075,
                ],
            ),
            (
                KEY_E,
                vec![
                    0x00CA, 0x00EA, 0x0000, 0x0000, 0x2045, 0x2065, 0x4045, 0x4065, 0x6045, 0x6065,
                    0x8045, 0x8065, 0xA045, 0xA065,
                ],
            ),
            (KEY_D, vec![0x0110, 0x0111]),
            (
                KEY_A | TONE_MASK,
                vec![
                    0x20C2, 0x20E2, 0x40C2, 0x40E2, 0x60C2, 0x60E2, 0x80C2, 0x80E2, 0xA0C2, 0xA0E2,
                ],
            ),
            (
                KEY_A | TONEW_MASK,
                vec![
                    0x2102, 0x2103, 0x4102, 0x4103, 0x6102, 0x6103, 0x8102, 0x8103, 0xA102, 0xA103,
                ],
            ),
            (
                KEY_O | TONE_MASK,
                vec![
                    0x20D4, 0x20F4, 0x40D4, 0x40F4, 0x60D4, 0x60F4, 0x80D4, 0x80F4, 0xA0D4, 0xA0F4,
                ],
            ),
            (
                KEY_O | TONEW_MASK,
                vec![
                    0x21A0, 0x21A1, 0x41A0, 0x41A1, 0x61A0, 0x61A1, 0x81A0, 0x81A1, 0xA1A0, 0xA1A1,
                ],
            ),
            (
                KEY_U | TONEW_MASK,
                vec![
                    0x21AF, 0x21B0, 0x41AF, 0x41B0, 0x61AF, 0x61B0, 0x81AF, 0x81B0, 0xA1AF, 0xA1B0,
                ],
            ),
            (
                KEY_E | TONE_MASK,
                vec![
                    0x20CA, 0x20EA, 0x40CA, 0x40EA, 0x60CA, 0x60EA, 0x80CA, 0x80EA, 0xA0CA, 0xA0EA,
                ],
            ),
            (
                KEY_I,
                vec![
                    0x2049, 0x2069, 0x4049, 0x4069, 0x6049, 0x6069, 0x8049, 0x8069, 0xA049, 0xA069,
                ],
            ),
            (
                KEY_Y,
                vec![
                    0x2059, 0x2079, 0x4059, 0x4079, 0x6059, 0x6079, 0x8059, 0x8079, 0xA059, 0xA079,
                ],
            ),
        ]),
        BTreeMap::from([
            (
                KEY_A,
                vec![
                    0x00C2, 0x00E2, 0x00C3, 0x00E3, 0xEC41, 0xEC61, 0xCC41, 0xCC61, 0xD241, 0xD261,
                    0xDE41, 0xDE61, 0xF241, 0xF261,
                ],
            ),
            (
                KEY_O,
                vec![
                    0x00D4, 0x00F4, 0x00D5, 0x00F5, 0xEC4F, 0xEC6F, 0xCC4F, 0xCC6F, 0xD24F, 0xD26F,
                    0xDE4F, 0xDE6F, 0xF24F, 0xF26F,
                ],
            ),
            (
                KEY_U,
                vec![
                    0x0000, 0x0000, 0x00DD, 0x00FD, 0xEC55, 0xEC75, 0xCC55, 0xCC75, 0xD255, 0xD275,
                    0xDE55, 0xDE75, 0xF255, 0xF275,
                ],
            ),
            (
                KEY_E,
                vec![
                    0x00CA, 0x00EA, 0x0000, 0x0000, 0xEC45, 0xEC65, 0xCC45, 0xCC65, 0xD245, 0xD265,
                    0xDE45, 0xDE65, 0xF245, 0xF265,
                ],
            ),
            (KEY_D, vec![0x00D0, 0x00F0]),
            (
                KEY_A | TONE_MASK,
                vec![
                    0xECC2, 0xECE2, 0xCCC2, 0xCCE2, 0xD2C2, 0xD2E2, 0xDEC2, 0xDEE2, 0xF2C2, 0xF2E2,
                ],
            ),
            (
                KEY_A | TONEW_MASK,
                vec![
                    0xECC3, 0xECE3, 0xCCC3, 0xCCE3, 0xD2C3, 0xD2E3, 0xDEC3, 0xDEE3, 0xF2C3, 0xF2E3,
                ],
            ),
            (
                KEY_O | TONE_MASK,
                vec![
                    0xECD4, 0xECF4, 0xCCD4, 0xCCF4, 0xD2D4, 0xD2F4, 0xDED4, 0xDEF4, 0xF2D4, 0xF2F4,
                ],
            ),
            (
                KEY_O | TONEW_MASK,
                vec![
                    0xECD5, 0xECF5, 0xCCD5, 0xCCF5, 0xD2D5, 0xD2F5, 0xDED5, 0xDEF5, 0xF2D5, 0xF2F5,
                ],
            ),
            (
                KEY_U | TONEW_MASK,
                vec![
                    0xECDD, 0xECFD, 0xCCDD, 0xCCFD, 0xD2DD, 0xD2FD, 0xDEDD, 0xDEFD, 0xF2DD, 0xF2FD,
                ],
            ),
            (
                KEY_E | TONE_MASK,
                vec![
                    0xECCA, 0xECEA, 0xCCCA, 0xCCEA, 0xD2CA, 0xD2EA, 0xDECA, 0xDEEA, 0xF2CA, 0xF2EA,
                ],
            ),
            (
                KEY_I,
                vec![
                    0xEC49, 0xEC69, 0xCC49, 0xCC69, 0xD249, 0xD269, 0xDE49, 0xDE69, 0xF249, 0xF269,
                ],
            ),
            (
                KEY_Y,
                vec![
                    0xEC59, 0xEC79, 0xCC59, 0xCC79, 0xD259, 0xD279, 0xDE59, 0xDE79, 0xF259, 0xF279,
                ],
            ),
        ]),
    ]
});
pub static _unicodeCompoundMark: LazyLock<Vec<i64>> =
    LazyLock::new(|| vec![0x0301, 0x0300, 0x0309, 0x0303, 0x0323]);
pub static _quickTelex: LazyLock<BTreeMap<i64, Vec<i64>>> = LazyLock::new(|| {
    BTreeMap::from([
        (KEY_C, vec![KEY_C, KEY_H]),
        (KEY_G, vec![KEY_G, KEY_I]),
        (KEY_K, vec![KEY_K, KEY_H]),
        (KEY_N, vec![KEY_N, KEY_G]),
        (KEY_Q, vec![KEY_Q, KEY_U]),
        (KEY_P, vec![KEY_P, KEY_H]),
        (KEY_T, vec![KEY_T, KEY_H]),
        (KEY_U, vec![KEY_U, KEY_U]),
    ])
});
pub static _characterMap: LazyLock<BTreeMap<i64, i64>> = LazyLock::new(|| {
    BTreeMap::from([
        (97, KEY_A),
        (65, KEY_A | CAPS_MASK),
        (98, KEY_B),
        (66, KEY_B | CAPS_MASK),
        (99, KEY_C),
        (67, KEY_C | CAPS_MASK),
        (100, KEY_D),
        (68, KEY_D | CAPS_MASK),
        (101, KEY_E),
        (69, KEY_E | CAPS_MASK),
        (102, KEY_F),
        (70, KEY_F | CAPS_MASK),
        (103, KEY_G),
        (71, KEY_G | CAPS_MASK),
        (104, KEY_H),
        (72, KEY_H | CAPS_MASK),
        (105, KEY_I),
        (73, KEY_I | CAPS_MASK),
        (106, KEY_J),
        (74, KEY_J | CAPS_MASK),
        (107, KEY_K),
        (75, KEY_K | CAPS_MASK),
        (108, KEY_L),
        (76, KEY_L | CAPS_MASK),
        (109, KEY_M),
        (77, KEY_M | CAPS_MASK),
        (110, KEY_N),
        (78, KEY_N | CAPS_MASK),
        (111, KEY_O),
        (79, KEY_O | CAPS_MASK),
        (112, KEY_P),
        (80, KEY_P | CAPS_MASK),
        (113, KEY_Q),
        (81, KEY_Q | CAPS_MASK),
        (114, KEY_R),
        (82, KEY_R | CAPS_MASK),
        (115, KEY_S),
        (83, KEY_S | CAPS_MASK),
        (116, KEY_T),
        (84, KEY_T | CAPS_MASK),
        (117, KEY_U),
        (85, KEY_U | CAPS_MASK),
        (118, KEY_V),
        (86, KEY_V | CAPS_MASK),
        (119, KEY_W),
        (87, KEY_W | CAPS_MASK),
        (120, KEY_X),
        (88, KEY_X | CAPS_MASK),
        (121, KEY_Y),
        (89, KEY_Y | CAPS_MASK),
        (122, KEY_Z),
        (90, KEY_Z | CAPS_MASK),
        (49, KEY_1),
        (33, KEY_1 | CAPS_MASK),
        (50, KEY_2),
        (64, KEY_2 | CAPS_MASK),
        (51, KEY_3),
        (35, KEY_3 | CAPS_MASK),
        (52, KEY_4),
        (36, KEY_4 | CAPS_MASK),
        (53, KEY_5),
        (37, KEY_5 | CAPS_MASK),
        (54, KEY_6),
        (94, KEY_6 | CAPS_MASK),
        (55, KEY_7),
        (38, KEY_7 | CAPS_MASK),
        (56, KEY_8),
        (42, KEY_8 | CAPS_MASK),
        (57, KEY_9),
        (40, KEY_9 | CAPS_MASK),
        (48, KEY_0),
        (41, KEY_0 | CAPS_MASK),
        (96, KEY_BACKQUOTE),
        (126, KEY_BACKQUOTE | CAPS_MASK),
        (45, KEY_MINUS),
        (95, KEY_MINUS | CAPS_MASK),
        (61, KEY_EQUALS),
        (43, KEY_EQUALS | CAPS_MASK),
        (91, KEY_LEFT_BRACKET),
        (123, KEY_LEFT_BRACKET | CAPS_MASK),
        (93, KEY_RIGHT_BRACKET),
        (125, KEY_RIGHT_BRACKET | CAPS_MASK),
        (92, KEY_BACK_SLASH),
        (124, KEY_BACK_SLASH | CAPS_MASK),
        (59, KEY_SEMICOLON),
        (58, KEY_SEMICOLON | CAPS_MASK),
        (39, KEY_QUOTE),
        (34, KEY_QUOTE | CAPS_MASK),
        (44, KEY_COMMA),
        (60, KEY_COMMA | CAPS_MASK),
        (46, KEY_DOT),
        (62, KEY_DOT | CAPS_MASK),
        (47, KEY_SLASH),
        (63, KEY_SLASH | CAPS_MASK),
        (32, KEY_SPACE),
    ])
});
pub fn key_code_to_character(key: u32) -> u16 {
    _characterMap
        .iter()
        .rev()
        .find(|(_, v)| **v == key as i64)
        .map(|(k, _)| *k as u16)
        .unwrap_or(0)
}
pub fn character_to_key(c: char) -> Option<u32> {
    _characterMap.get(&(c as i64)).map(|v| *v as u32)
}
const CHARACTER_KEYS: &[i64] = &[
    KEY_BACKQUOTE,
    KEY_1,
    KEY_2,
    KEY_3,
    KEY_4,
    KEY_5,
    KEY_6,
    KEY_7,
    KEY_8,
    KEY_9,
    KEY_0,
    KEY_MINUS,
    KEY_EQUALS,
    KEY_LEFT_BRACKET,
    KEY_RIGHT_BRACKET,
    KEY_BACK_SLASH,
    KEY_SEMICOLON,
    KEY_QUOTE,
    KEY_COMMA,
    KEY_DOT,
    KEY_SLASH,
];
pub static _charKeyCode: LazyLock<Vec<i64>> = LazyLock::new(|| CHARACTER_KEYS.to_vec());
const WORD_BREAK_KEYS: &[i64] = &[
    KEY_ESC,
    KEY_TAB,
    KEY_ENTER,
    KEY_RETURN,
    KEY_LEFT,
    KEY_RIGHT,
    KEY_DOWN,
    KEY_UP,
    KEY_COMMA,
    KEY_DOT,
    KEY_SLASH,
    KEY_SEMICOLON,
    KEY_QUOTE,
    KEY_BACK_SLASH,
    KEY_MINUS,
    KEY_EQUALS,
    KEY_BACKQUOTE,
    KEY_TAB,
];
pub static _breakCode: LazyLock<Vec<i64>> = LazyLock::new(|| WORD_BREAK_KEYS.to_vec());
const MACRO_BREAK_KEYS: &[i64] = &[
    KEY_RETURN,
    KEY_COMMA,
    KEY_DOT,
    KEY_SLASH,
    KEY_SEMICOLON,
    KEY_QUOTE,
    KEY_BACK_SLASH,
    KEY_MINUS,
    KEY_EQUALS,
];
pub static _macroBreakCode: LazyLock<Vec<i64>> = LazyLock::new(|| MACRO_BREAK_KEYS.to_vec());
pub(crate) const CHARACTER_KEY: u8 = 1;
pub(crate) const WORD_BREAK_KEY: u8 = 2;
pub(crate) const MACRO_BREAK_KEY: u8 = 4;

const KEY_CLASSES: [u8; 128] = {
    let mut classes = [0; 128];
    let groups = [
        (CHARACTER_KEYS, CHARACTER_KEY),
        (WORD_BREAK_KEYS, WORD_BREAK_KEY),
        (MACRO_BREAK_KEYS, MACRO_BREAK_KEY),
    ];
    let mut group = 0;
    while group < groups.len() {
        let (keys, class) = groups[group];
        let mut i = 0;
        while i < keys.len() {
            classes[keys[i] as usize] |= class;
            i += 1;
        }
        group += 1;
    }
    classes
};

pub(crate) fn key_classes(key: i64) -> u8 {
    KEY_CLASSES.get(key as usize).copied().unwrap_or(0)
}

pub static ProcessingChar: LazyLock<Vec<Vec<i64>>> = LazyLock::new(|| {
    vec![
        vec![
            KEY_S, KEY_F, KEY_R, KEY_X, KEY_J, KEY_A, KEY_O, KEY_E, KEY_W, KEY_D, KEY_Z,
        ],
        vec![
            KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_6, KEY_7, KEY_8, KEY_9, KEY_0,
        ],
        vec![
            KEY_S, KEY_F, KEY_R, KEY_X, KEY_J, KEY_A, KEY_O, KEY_E, KEY_W, KEY_D, KEY_Z,
        ],
        vec![
            KEY_S, KEY_F, KEY_R, KEY_X, KEY_J, KEY_A, KEY_O, KEY_E, KEY_W, KEY_D, KEY_Z,
        ],
    ]
});
