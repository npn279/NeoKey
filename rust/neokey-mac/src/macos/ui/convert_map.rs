//! Map UI case-mode choices onto `ConvertOptions`.

use neokey_engine::ConvertOptions;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseMode {
    Keep,
    Upper,
    Lower,
    Sentence,
    Title,
}

impl CaseMode {
    pub const ALL: [CaseMode; 5] = [
        CaseMode::Keep,
        CaseMode::Upper,
        CaseMode::Lower,
        CaseMode::Sentence,
        CaseMode::Title,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Keep => "Giữ nguyên",
            Self::Upper => "VIẾT HOA",
            Self::Lower => "viết thường",
            Self::Sentence => "Hoa đầu câu",
            Self::Title => "Hoa Mỗi Từ",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConvertUiOptions {
    pub from_code: u8,
    pub to_code: u8,
    pub case_mode: CaseMode,
    pub remove_mark: bool,
}

impl ConvertUiOptions {
    pub fn to_engine(self) -> ConvertOptions {
        ConvertOptions {
            dont_alert_when_completed: true,
            to_all_caps: self.case_mode == CaseMode::Upper,
            to_all_non_caps: self.case_mode == CaseMode::Lower,
            to_caps_first_letter: self.case_mode == CaseMode::Sentence,
            to_caps_each_word: self.case_mode == CaseMode::Title,
            remove_mark: self.remove_mark,
            from_code: self.from_code.min(4),
            to_code: self.to_code.min(4),
            hot_key: 0,
        }
    }

    pub fn convert(self, source: &str) -> String {
        self.to_engine()
            .convert(source)
            .unwrap_or_else(|_| source.to_string())
    }

    pub fn swap_tables(mut self) -> Self {
        std::mem::swap(&mut self.from_code, &mut self.to_code);
        self
    }
}

pub const CODE_TABLE_LABELS: [&str; 5] = [
    "Unicode",
    "TCVN3 (ABC)",
    "VNI Windows",
    "Unicode tổ hợp",
    "Vietnamese Locale CP 1258",
];

pub const INPUT_TYPE_LABELS: [&str; 4] = ["Telex", "VNI", "Telex đơn", "Telex đơn 2"];

pub fn input_type_hint(mode: i64) -> &'static str {
    match mode {
        1 => "a6 → â, o7 → ơ, 1–5 → sắc huyền hỏi ngã nặng",
        2 | 3 => "Như Telex nhưng không tự biến w thành ư",
        _ => "aa → â, ow → ơ, s f r x j → sắc huyền hỏi ngã nặng",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_modes_set_exactly_one_engine_flag() {
        let keep = ConvertUiOptions {
            from_code: 0,
            to_code: 0,
            case_mode: CaseMode::Keep,
            remove_mark: false,
        }
        .to_engine();
        assert!(!keep.to_all_caps && !keep.to_all_non_caps);
        assert!(!keep.to_caps_first_letter && !keep.to_caps_each_word);

        let upper = ConvertUiOptions {
            case_mode: CaseMode::Upper,
            ..ConvertUiOptions {
                from_code: 0,
                to_code: 0,
                case_mode: CaseMode::Keep,
                remove_mark: false,
            }
        }
        .to_engine();
        assert!(upper.to_all_caps);
        assert!(!upper.to_all_non_caps);

        let title = ConvertUiOptions {
            from_code: 0,
            to_code: 0,
            case_mode: CaseMode::Title,
            remove_mark: true,
        }
        .to_engine();
        assert!(title.to_caps_each_word);
        assert!(title.remove_mark);
    }

    #[test]
    fn swap_tables_exchanges_from_and_to() {
        let options = ConvertUiOptions {
            from_code: 1,
            to_code: 3,
            case_mode: CaseMode::Keep,
            remove_mark: false,
        }
        .swap_tables();
        assert_eq!((options.from_code, options.to_code), (3, 1));
    }

    #[test]
    fn convert_upper_ascii_path_matches_engine() {
        let out = ConvertUiOptions {
            from_code: 0,
            to_code: 0,
            case_mode: CaseMode::Upper,
            remove_mark: false,
        }
        .convert("ab");
        assert_eq!(out, "AB");
    }
}
