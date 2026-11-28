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
            Self::Keep => "Mặc định",
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
        2 => "Như Telex, nhưng w đứng riêng giữ nguyên là w; không dùng [ ]",
        3 => "Như Telex (w đứng riêng → ư), chỉ bỏ [ → ơ và ] → ư",
        _ => "aa → â, ow → ơ, s f r x j → sắc huyền hỏi ngã nặng",
    }
}

/// Longer per-type descriptions shown as tooltips on the input-type pickers.
/// The two Simple Telex modes differ only in how a standalone `w` is handled
/// (`Engine.cpp` / `composition.rs`: `vInputType != vSimpleTelex1`).
/// Full help for the "Kiểu gõ" picker: every type, so they can be compared.
pub fn input_type_help() -> String {
    let mut text = String::from("Cách gõ dấu tiếng Việt. Khác nhau giữa các kiểu:");
    for (name, tip) in INPUT_TYPE_LABELS.iter().zip(INPUT_TYPE_TOOLTIPS) {
        let tip = tip.strip_prefix(&format!("{name}: ")).unwrap_or(tip);
        text.push_str(&format!("\n\n• {name}: {}", tip.replace('\n', " ")));
    }
    text
}

pub const INPUT_TYPE_TOOLTIPS: [&str; 4] = [
    "Telex đầy đủ: aa → â, aw → ă, ow → ơ, uw → ư, dd → đ, w → ư, [ → ơ, ] → ư.\n\
     Dấu: s f r x j = sắc huyền hỏi ngã nặng, z xoá dấu.",
    "VNI: 6 → â ê ô, 7 → ơ ư, 8 → ă, 9 → đ.\n\
     Dấu: 1–5 = sắc huyền hỏi ngã nặng, 0 xoá dấu.",
    "Telex đơn: như Telex nhưng w đứng riêng KHÔNG thành ư (tw → tw),\n\
     chỉ aw, ow, uw mới thành ă, ơ, ư. [ ] là dấu ngoặc bình thường.\n\
     Hợp khi hay gõ tiếng Anh, tên riêng hoặc code có chữ w.",
    "Telex đơn 2: như Telex, w đứng riêng vẫn thành ư (tw → tư).\n\
     Chỉ bỏ [ → ơ và ] → ư; [ ] là dấu ngoặc bình thường.",
];

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

#[cfg(test)]
mod help_tests {
    use super::*;

    #[test]
    fn input_type_help_lists_every_type_once() {
        let help = input_type_help();
        for name in INPUT_TYPE_LABELS {
            assert!(help.contains(&format!("• {name}: ")), "{name} missing");
        }
        // The per-type prefix ("Telex đơn: …") is not repeated after the bullet.
        assert!(!help.contains("Telex đơn: Telex đơn"));
        assert!(!help.contains('\n') || help.lines().all(|line| !line.starts_with(' ')));
    }
}
