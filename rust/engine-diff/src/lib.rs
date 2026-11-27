//! Differential oracle: the reference implementation is the original C++ source.
#![deny(unsafe_code)]
#[allow(unsafe_code)]
mod ffi;
use neokey_engine::{
    data::*,
    vietnamese::{character_to_key, key_code_to_character},
    ConvertOptions, Engine, Event, EventState, HookState, Settings, SmartSwitch,
};

pub const CORPUS:&str="Tiếng Việt rất giàu và đẹp. Tôi yêu quê hương đất nước Việt Nam!\nChúng ta cùng học tiếng Việt, giữ gìn bản sắc văn hóa dân tộc.\nTrường đại học công nghệ thông tin ở thành phố Hồ Chí Minh.\nCộng hòa xã hội chủ nghĩa Việt Nam: Độc lập, Tự do, Hạnh phúc.\nNgười nông dân trồng lúa trên cánh đồng; những đứa trẻ vui chơi dưới ánh nắng.\nHòa bình, thủy chung, quyển sách, khuya, giếng, nghiêng, Nguyễn, Đặng, ươm, ướt, khuỷu, ngoằn ngoèo.\nÁ À Ả Ã Ạ Ấ Ầ Ẩ Ẫ Ậ Ắ Ằ Ẳ Ẵ Ặ É È Ẻ Ẽ Ẹ Ế Ề Ể Ễ Ệ Í Ì Ỉ Ĩ Ị Ó Ò Ỏ Õ Ọ Ố Ồ Ổ Ỗ Ộ Ớ Ờ Ở Ỡ Ợ Ú Ù Ủ Ũ Ụ Ứ Ừ Ử Ữ Ự Ý Ỳ Ỷ Ỹ Ỵ đ Đ.\nXin chào! Cảm ơn bạn. Hẹn gặp lại vào ngày mai. 2026, 12345. btw BTW Btw vn dc.\n";
const FAMILIES: [(&str, &str); 12] = [
    ("aáàảãạ", "a"),
    ("âấầẩẫậ", "aa"),
    ("ăắằẳẵặ", "aw"),
    ("eéèẻẽẹ", "e"),
    ("êếềểễệ", "ee"),
    ("iíìỉĩị", "i"),
    ("oóòỏõọ", "o"),
    ("ôốồổỗộ", "oo"),
    ("ơớờởỡợ", "ow"),
    ("uúùủũụ", "u"),
    ("ưứừửữự", "uw"),
    ("yýỳỷỹỵ", "y"),
];
/// Generate conventional end-of-syllable tone keystrokes from Vietnamese text.
/// Both Simple Telex variants use the same spelling keys as Telex.
pub fn vietnamese_keystrokes(text: &str, input_type: i64) -> String {
    let mut out = String::new();
    let mut word = String::new();
    let mut tone = 0;
    let flush = |out: &mut String, word: &mut String, tone: &mut usize| {
        // A shared horn on ươ is typed with one w/7.
        *word = word
            .replace("uwow", "uow")
            .replace("uWoW", "uoW")
            .replace("UWOW", "UOW")
            .replace("u7o7", "uo7")
            .replace("U7O7", "UO7");
        out.push_str(word);
        word.clear();
        if *tone > 0 {
            out.push(if input_type == 1 {
                [b'0', b'1', b'2', b'3', b'4', b'5'][*tone] as char
            } else {
                [b'z', b's', b'f', b'r', b'x', b'j'][*tone] as char
            });
        }
        *tone = 0;
    };
    for c in text.chars() {
        if !c.is_alphabetic() {
            flush(&mut out, &mut word, &mut tone);
            out.push(c);
            continue;
        }
        let lower = c.to_lowercase().next().unwrap();
        let upper = c.is_uppercase();
        let mut spelling = lower.to_string();
        if lower == 'đ' {
            spelling = if input_type == 1 { "d9" } else { "dd" }.into();
        }
        for (family, keys) in FAMILIES {
            if let Some(n) = family.chars().position(|v| v == lower) {
                if n > 0 {
                    tone = n;
                }
                spelling = if input_type == 1 {
                    match keys {
                        "aa" => "a6",
                        "aw" => "a8",
                        "ee" => "e6",
                        "oo" => "o6",
                        "ow" => "o7",
                        "uw" => "u7",
                        _ => keys,
                    }
                    .into()
                } else {
                    keys.into()
                };
                break;
            }
        }
        word.push_str(&if upper {
            spelling.to_uppercase()
        } else {
            spelling
        });
    }
    flush(&mut out, &mut word, &mut tone);
    out
}
#[derive(Clone, Copy, Debug)]
pub struct Key {
    pub data: u16,
    pub caps: u8,
    pub event: Event,
    pub state: EventState,
    pub ctrl: bool,
}
impl Key {
    pub fn character(c: char) -> Self {
        let code = if c == '\n' {
            KEY_RETURN as u32
        } else {
            character_to_key(c).unwrap_or_else(|| panic!("unmapped corpus character {c:?}"))
        };
        Self {
            data: (code & 0xffff) as u16,
            caps: if code & CAPS_MASK as u32 != 0 { 1 } else { 0 },
            event: Event::Keyboard,
            state: EventState::KeyDown,
            ctrl: false,
        }
    }
}
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}
#[derive(Default, PartialEq, Eq, Debug)]
struct Screen(Vec<Vec<u16>>);
impl Screen {
    fn apply(&mut self, h: &HookState, k: Key, table: usize, reference: bool) {
        if !matches!(k.event, Event::Keyboard) || !matches!(k.state, EventState::KeyDown) || k.ctrl
        {
            return;
        }
        let original = k.data as u32
            | if k.caps == 1 || k.caps == 2 {
                CAPS_MASK as u32
            } else {
                0
            };
        let key_char = |v| {
            if reference {
                ffi::key_character(v)
            } else {
                key_code_to_character(v)
            }
        };
        let decode = |v: u32| -> Vec<u16> {
            let packed = if reference { ffi::character(v) } else { v };
            if packed & PURE_CHARACTER_MASK as u32 != 0 {
                return vec![packed as u16];
            }
            if packed & CHAR_CODE_MASK as u32 == 0 {
                return vec![key_char(packed)];
            }
            let c = packed as u16;
            match table {
                1 | 2 | 4 => {
                    let mut v = vec![c & 255];
                    if c >> 8 > 32 {
                        v.push(c >> 8);
                    }
                    v
                }
                3 => {
                    let mut v = vec![c & 0x1fff];
                    if c >> 13 > 0 {
                        v.push([0x301, 0x300, 0x309, 0x303, 0x323][((c >> 13) - 1) as usize]);
                    }
                    v
                }
                _ => vec![c],
            }
        };
        if [1, 3, 4, 5].contains(&h.code) {
            for _ in 0..h.backspace_count {
                self.0.pop();
            }
            if h.code == 4 {
                for c in &h.macro_data {
                    self.0.push(decode(*c));
                }
            } else {
                for c in h.char_data[..h.new_char_count as usize].iter().rev() {
                    self.0.push(decode(*c));
                }
            }
            if h.code != 1 {
                let c = key_char(original);
                if c != 0 {
                    self.0.push(vec![c]);
                }
            }
        } else if k.data == KEY_DELETE as u16 {
            self.0.pop();
        } else if k.data == KEY_RETURN as u16 {
            self.0.push(vec![10]);
        } else {
            let c = key_char(original);
            if c != 0 {
                self.0.push(vec![c]);
            }
        }
    }
}
fn compare(
    rust: &HookState,
    cpp: &HookState,
    settings: &Settings,
    index: usize,
    k: Key,
    history: &[Key],
) {
    let n = cpp.new_char_count as usize;
    let same = rust.code == cpp.code
        && rust.backspace_count == cpp.backspace_count
        && rust.new_char_count == cpp.new_char_count
        && rust.ext_code == cpp.ext_code
        && n <= 32
        && rust.char_data == cpp.char_data
        && rust.macro_data == cpp.macro_data
        && rust.macro_key == cpp.macro_key;
    assert!(same,"mismatch at {index}, key={k:?}\nsettings={settings:?}\nrust={rust:?}\ncpp={cpp:?}\nlast keys={:?}",&history[index.saturating_sub(48)..=index]);
}
fn run_case(settings: Settings, keys: &[Key]) -> usize {
    ffi::settings(&settings);
    ffi::reset();
    let mut engine = Engine::with_settings(settings.clone());
    for (k, v) in benchmark::MACROS {
        engine.add_macro(k, v);
        ffi::add_macro(k, v);
    }
    let mut a = Screen::default();
    let mut b = Screen::default();
    for (i, k) in keys.iter().copied().enumerate() {
        let r = if settings.vLanguage == 0 {
            engine.english_mode(k.state, k.data, k.caps != 0, k.ctrl)
        } else {
            engine.handle_event(k.event, k.state, k.data, k.caps, k.ctrl)
        }
        .clone();
        if settings.vLanguage == 0 {
            ffi::english(k.state as u8, k.data, k.caps != 0, k.ctrl);
        } else {
            ffi::event(k.event as u8, k.state as u8, k.data, k.caps, k.ctrl);
        }
        let c = ffi::hook();
        compare(&r, &c, &settings, i, k, keys);
        // Decode the Rust side through the Rust decoder as well as comparing packed hook values.
        let mut decoded = r.clone();
        for d in &mut decoded.char_data {
            *d = engine.get_character_code(*d);
        }
        for d in &mut decoded.macro_data {
            *d = engine.get_character_code(*d);
        }
        a.apply(&decoded, k, settings.vCodeTable as usize, false);
        b.apply(&c, k, settings.vCodeTable as usize, true);
        assert_eq!(a, b, "screen mismatch at {i}: {settings:?}");
        // The platform starts a fresh session after this output has been applied.
        if r.code == 5 {
            engine.start_new_session();
            ffi::session();
        }
    }
    assert_eq!(a, b);
    keys.len()
}
fn settings(input: i64, table: i64, profile: u64) -> Settings {
    let mut s = Settings {
        vInputType: input,
        vCodeTable: table,
        ..Settings::default()
    };
    let fields = [
        &mut s.vCheckSpelling,
        &mut s.vUseModernOrthography,
        &mut s.vQuickTelex,
        &mut s.vFreeMark,
        &mut s.vRestoreIfWrongSpelling,
        &mut s.vUseMacro,
        &mut s.vUpperCaseFirstChar,
        &mut s.vQuickStartConsonant,
        &mut s.vQuickEndConsonant,
        &mut s.vAllowConsonantZFWJ,
        &mut s.vAutoCapsMacro,
        &mut s.vFixRecommendBrowser,
    ];
    for (i, v) in fields.into_iter().enumerate() {
        *v = ((profile >> i) & 1) as i64;
    }
    s
}
fn utility_checks() {
    let mut engine = Engine::new();
    for table in 0..5 {
        let s = Settings {
            vCodeTable: table,
            ..Settings::default()
        };
        engine.set_settings(s.clone());
        ffi::settings(&s);
        for key in 0..=50u32 {
            for caps in [0, CAPS_MASK as u32] {
                for shape in [0, TONE_MASK as u32, TONEW_MASK as u32] {
                    for mark in [
                        0,
                        MARK1_MASK as u32,
                        MARK2_MASK as u32,
                        MARK3_MASK as u32,
                        MARK4_MASK as u32,
                        MARK5_MASK as u32,
                    ] {
                        if key == KEY_D as u32 && (mark != 0 || shape == TONEW_MASK as u32) {
                            continue;
                        }
                        if [KEY_I as u32, KEY_Y as u32].contains(&key) && shape != 0 {
                            continue;
                        }
                        let v = key | caps | shape | mark;
                        assert_eq!(
                            engine.get_character_code(v),
                            ffi::character(v),
                            "decoder table={table} packed={v:x}"
                        );
                    }
                }
            }
        }
    }

    for from in 0..5 {
        for to in 0..5 {
            for flags in 0..32 {
                let initial = ConvertOptions {
                    to_code: from,
                    ..ConvertOptions::default()
                };
                let source = ffi::convert(CORPUS, &initial).unwrap();
                let o = ConvertOptions {
                    from_code: from,
                    to_code: to,
                    to_all_caps: flags & 1 != 0,
                    to_all_non_caps: flags & 2 != 0,
                    to_caps_first_letter: flags & 4 != 0,
                    to_caps_each_word: flags & 8 != 0,
                    remove_mark: flags & 16 != 0,
                    ..ConvertOptions::default()
                };
                assert_eq!(
                    o.convert(&source),
                    ffi::convert(&source, &o),
                    "convert from={from} to={to} flags={flags}"
                );
            }
        }
    }
    let mut smart = SmartSwitch::default();
    ffi::init_smart(&[0, 0]);
    for (key, v) in [
        ("com.apple.Terminal", 1),
        ("org.editor", 0),
        ("vietnamese", 9),
        ("", 0),
    ] {
        assert_eq!(smart.get(key, v), ffi::get_smart(key, v));
        smart.set(key, v);
        ffi::set_smart(key, v);
        assert_eq!(smart.get(key, 0), ffi::get_smart(key, 0));
    }
    let bytes = smart.save_data();
    assert_eq!(bytes, ffi::save_smart());
    smart.init(&bytes).unwrap();
    ffi::init_smart(&bytes);
    assert_eq!(smart.save_data(), ffi::save_smart());
    for table in 0..5 {
        let s = Settings {
            vCodeTable: table,
            ..Settings::default()
        };
        ffi::settings(&s);
        ffi::reset();
        let mut e = Engine::with_settings(s);
        for (k, v) in [
            ("vn", "Việt Nam"),
            ("a", "hello"),
            ("🙂", "🦀"),
            ("vn", "VIỆT NAM"),
        ] {
            e.add_macro(k, v);
            ffi::add_macro(k, v);
        }
        let bytes = e.macros().save_data();
        assert_eq!(bytes, ffi::save_macro());
        e.init_macros(&bytes).unwrap();
        ffi::init_macro(&bytes);
        assert_eq!(e.macros().save_data(), ffi::save_macro());
        assert_eq!(e.delete_macro("vn"), ffi::delete_macro("vn"));
        assert_eq!(e.macros().save_data(), ffi::save_macro());
    }
    // Binary macro records may contain NUL: C++ encoding stops at it, serialization retains it.
    let s = Settings {
        vUseMacro: 1,
        ..Settings::default()
    };
    ffi::settings(&s);
    ffi::reset();
    let mut e = Engine::with_settings(s.clone());
    let bytes = [1, 0, 3, b'a', 0, b'b', 3, 0, b'x', 0, b'y'];
    e.init_macros(&bytes).unwrap();
    ffi::init_macro(&bytes);
    assert_eq!(e.macros().save_data(), ffi::save_macro());
    let seq: Vec<_> = "a ".chars().map(Key::character).collect();
    for (i, k) in seq.iter().copied().enumerate() {
        let r = e
            .handle_event(k.event, k.state, k.data, k.caps, k.ctrl)
            .clone();
        ffi::event(k.event as u8, k.state as u8, k.data, k.caps, k.ctrl);
        compare(&r, &ffi::hook(), &s, i, k, &seq);
    }
    // Explicit temporary toggles, session boundaries and English mode are observable API paths.
    let s = Settings {
        vUseMacro: 1,
        vCheckSpelling: 1,
        ..Settings::default()
    };
    ffi::settings(&s);
    ffi::reset();
    let mut e = Engine::with_settings(s.clone());
    let seq: Vec<Key> = "tieengs vieetj abc ".chars().map(Key::character).collect();
    for (i, k) in seq.iter().copied().enumerate() {
        if i == 3 || i == 8 {
            e.temp_off_spell_checking();
            ffi::temp_spell();
        }
        if i == 10 || i == 14 {
            e.temp_off_engine(i == 10);
            ffi::temp_engine(i == 10);
        }
        let r = e
            .handle_event(k.event, k.state, k.data, k.caps, k.ctrl)
            .clone();
        ffi::event(k.event as u8, k.state as u8, k.data, k.caps, k.ctrl);
        compare(&r, &ffi::hook(), &s, i, k, &seq);
    }
}
// Distinct parity masks over six bits cover all four values of every settings pair.
// Profiles 0 and 1 additionally force all settings off/on.
fn profile_bits(profile: usize) -> u64 {
    if profile == 0 {
        return 0;
    }
    if profile == 1 {
        return 4095;
    }
    [1usize, 2, 4, 8, 16, 32, 3, 5, 9, 17, 33, 6]
        .into_iter()
        .enumerate()
        .fold(0, |bits, (i, mask)| {
            bits | (((profile & mask).count_ones() as u64 & 1) << i)
        })
}
/// Runs all input modes, all five code tables and mixed setting profiles.
/// A fixed seed makes the first failing sequence exactly reproducible.

fn random_keys(rng: &mut Rng, count: usize) -> impl Iterator<Item = Key> + '_ {
    let alphabet = b"abcdefghijklmnopqrstuvwxyz0123456789     .,!?;:'[]-=/\\`";
    (0..count).map(|_| {
        let v = rng.next();
        let mut k = Key::character(alphabet[(v as usize) % alphabet.len()] as char);
        if v % 11 == 0 {
            k.data = KEY_DELETE as u16;
        }
        k.caps = match (v >> 16) % 12 {
            0 => 1,
            1 => 2,
            _ => 0,
        };
        if v % 151 == 0 {
            k.ctrl = true;
        }
        if v % 251 == 0 {
            k.event = Event::Mouse;
            k.state = EventState::MouseDown;
        }
        if v % 263 == 0 {
            k.state = EventState::KeyUp;
        }
        k
    })
}

pub fn run(profiles: usize, random_per_case: usize) -> usize {
    utility_checks();
    let mut total = 0;
    let mut rng = Rng(0x4e656f4b6579);
    for input in 0..4 {
        let corpus = vietnamese_keystrokes(CORPUS, input);
        for table in 0..5 {
            for profile in 0..profiles {
                let bits = profile_bits(profile);
                let mut keys: Vec<Key> = corpus.chars().map(Key::character).collect();
                for text in [
                    "cc gg kk nn qq pp tt uu thuwowngf uowngf asss aaawww ddd ",
                    "[ ] [[ ]] . xin chao. ban khoe? btw BTW Btw vn dc sig ",
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa ",
                ] {
                    keys.extend(text.chars().map(Key::character));
                }
                // Long, continuous seeded streams include word/history editing, not just isolated words.
                keys.extend(random_keys(&mut rng, random_per_case));
                total += run_case(settings(input, table, bits), &keys);
            }
        }
    }
    for input in 0..4 {
        let mut s = settings(input, 0, 4095);
        s.vLanguage = 0;
        s.vUseMacroInEnglishMode = 1;
        let keys: Vec<_> = "btw BTW Btw vn dc sig  abc\n"
            .repeat(10)
            .chars()
            .map(Key::character)
            .collect();
        total += run_case(s, &keys);
    }
    total
}
/// Shared fixtures and single-threaded C++ oracle for the benchmark executable.
pub mod benchmark {
    use super::*;

    pub const MACROS: [(&str, &str); 4] = [
        ("btw", "nhân tiện"),
        ("vn", "Việt Nam"),
        ("dc", "được"),
        ("sig", "NeoKey 🦀"),
    ];

    pub fn random(count: usize) -> Vec<Key> {
        random_keys(&mut Rng(0x4e656f4b6579), count).collect()
    }

    pub fn verify(settings: Settings, keys: &[Key]) -> usize {
        run_case(settings, keys)
    }

    pub fn reference_setup(settings: &Settings) {
        ffi::settings(settings);
        ffi::reset();
        for (key, value) in MACROS {
            ffi::add_macro(key, value);
        }
    }

    pub fn reference_key(key: Key) -> u8 {
        let code = ffi::event(
            key.event as u8,
            key.state as u8,
            key.data,
            key.caps,
            key.ctrl,
        );
        if i64::from(code) == vRestoreAndStartNewSession {
            ffi::session();
        }
        code
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_cpp_matches_all_engine_paths() {
        assert!(run(4, 256) > 50_000);
    }
    #[test]
    fn settings_profiles_cover_every_binary_pair() {
        let profiles: Vec<_> = (0..64).map(profile_bits).collect();
        assert_eq!(
            profiles
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            64
        );
        for i in 0..12 {
            for j in i + 1..12 {
                let values = profiles.iter().fold(0, |mask, bits| {
                    mask | (1 << (((bits >> i) & 1) * 2 + ((bits >> j) & 1)))
                });
                assert_eq!(values, 15, "settings bits {i}/{j}");
            }
        }
    }
    #[test]
    fn corpus_generator_emits_vietnamese_typing_keys() {
        assert_eq!(vietnamese_keystrokes("Tiếng Việt", 0), "Tieengs Vieetj");
        assert_eq!(vietnamese_keystrokes("Tiếng Việt", 1), "Tie6ng1 Vie6t5");
    }
}
