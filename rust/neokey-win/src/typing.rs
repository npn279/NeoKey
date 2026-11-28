use crate::settings::{Hotkey, Preferences, Setting, ALT, CTRL, SHIFT, WIN};
use neokey_engine::data::*;
use neokey_engine::vietnamese::{_unicodeCompoundMark, key_code_to_character};
use neokey_engine::{Engine, Event, EventState, SmartSwitch};
use std::collections::VecDeque;

// The OEM punctuation values include the Win32 port's historical mappings.
const KEYS: &[(u16, i64)] = &[
    (0x08, KEY_DELETE),
    (0x09, KEY_TAB),
    (0x0d, KEY_RETURN),
    (0x1b, KEY_ESC),
    (0x20, KEY_SPACE),
    (0x25, KEY_LEFT),
    (0x26, KEY_UP),
    (0x27, KEY_RIGHT),
    (0x28, KEY_DOWN),
    (0x30, KEY_0),
    (0x31, KEY_1),
    (0x32, KEY_2),
    (0x33, KEY_3),
    (0x34, KEY_4),
    (0x35, KEY_5),
    (0x36, KEY_6),
    (0x37, KEY_7),
    (0x38, KEY_8),
    (0x39, KEY_9),
    (0x41, KEY_A),
    (0x42, KEY_B),
    (0x43, KEY_C),
    (0x44, KEY_D),
    (0x45, KEY_E),
    (0x46, KEY_F),
    (0x47, KEY_G),
    (0x48, KEY_H),
    (0x49, KEY_I),
    (0x4a, KEY_J),
    (0x4b, KEY_K),
    (0x4c, KEY_L),
    (0x4d, KEY_M),
    (0x4e, KEY_N),
    (0x4f, KEY_O),
    (0x50, KEY_P),
    (0x51, KEY_Q),
    (0x52, KEY_R),
    (0x53, KEY_S),
    (0x54, KEY_T),
    (0x55, KEY_U),
    (0x56, KEY_V),
    (0x57, KEY_W),
    (0x58, KEY_X),
    (0x59, KEY_Y),
    (0x5a, KEY_Z),
    (0xa0, KEY_LEFT_SHIFT),
    (0xa1, KEY_RIGHT_SHIFT),
    (0xba, KEY_SEMICOLON),
    (0xbb, KEY_EQUALS),
    (0xbc, KEY_COMMA),
    (0xbd, KEY_MINUS),
    (0xbe, KEY_DOT),
    (0xbf, KEY_SLASH),
    (0xc0, KEY_QUOTE),
    (0xdb, KEY_LEFT_BRACKET),
    (0xdc, KEY_BACKQUOTE),
    (0xdd, KEY_RIGHT_BRACKET),
    (0xde, KEY_BACK_SLASH),
];

pub fn capture_key(vk: u16) -> Option<u8> {
    // Backspace and Delete clear the shortcut. Other keys must be characters the
    // original dialog accepted through keyCodeToCharacter.
    if matches!(vk, 0x08 | 0x2e) {
        return Some(Hotkey::MODIFIERS_ONLY);
    }
    (vk <= u8::MAX.into() && key_code_to_character(u32::from(to_engine(vk))) != 0)
        .then_some(vk as u8)
}

pub fn to_engine(vk: u16) -> u16 {
    // The original Windows engine recognized additional navigation keys as word breaks.
    // Escape has the same engine class. Restores replay the original physical key.
    if matches!(vk, 0x21..=0x24 | 0x29..=0x2f) {
        return KEY_ESC as u16;
    }
    KEYS.binary_search_by_key(&vk, |entry| entry.0)
        .map_or(KEY_EMPTY, |index| KEYS[index].1) as u16
}

#[derive(Debug, PartialEq, Eq)]
pub enum Character {
    Text([u16; 2], u8),
    Key(u16),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Workaround {
    None,
    Placeholder,
    SelectLeft,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Edit {
    pub backspaces: usize,
    pub characters: Vec<Character>,
    pub macro_trigger: Option<Character>,
    pub workaround: Workaround,
    pub step_by_step: bool,
    pub metro: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Pass,
    Suppress,
    ExtraBackspace,
    Replace(Edit),
    ToggleLanguage,
    QuickConvert,
}

pub struct Typing {
    pub engine: Engine,
    pub preferences: Preferences,
    pub smart_switch: SmartSwitch,
    app: String,
    widths: VecDeque<u8>,
    modifiers: [bool; 8],
    chord_used: bool,
    hotkey_down: [bool; 256],
    spelling_checkbox: i64,
}

impl Typing {
    pub fn new(preferences: Preferences) -> Self {
        let engine = Engine::with_settings(preferences.engine());
        let spelling_checkbox = engine.settings().vCheckSpelling;
        Self {
            engine,
            preferences,
            smart_switch: SmartSwitch::default(),
            app: String::new(),
            widths: VecDeque::with_capacity(255),
            modifiers: [false; 8],
            chord_used: false,
            hotkey_down: [false; 256],
            spelling_checkbox,
        }
    }

    pub fn refresh_settings(&mut self) {
        let settings = self.preferences.engine();
        let checkbox = settings.vCheckSpelling;
        // Ctrl can turn spelling off while the checkbox stays on. Applying the
        // checkbox again would cancel that, so put the temporary value back
        // after the engine rebuilds its baseline.
        let restore_temporary_spelling = checkbox == self.spelling_checkbox
            && checkbox != 0
            && self.engine.settings().vCheckSpelling == 0;
        if self.engine.settings() != &settings {
            self.engine.set_settings(settings);
            if restore_temporary_spelling {
                self.engine.temp_off_spell_checking();
            }
            // A mouse event would also cancel a temporary Alt stop. The original
            // language switch only starts a new session.
            self.engine.start_new_session();
        }
        self.spelling_checkbox = checkbox;
    }

    pub fn reset(&mut self) {
        self.mouse_button();
        self.engine.start_new_session();
    }

    pub fn mouse_button(&mut self) {
        self.engine
            .handle_event(Event::Mouse, EventState::MouseDown, 0, 0, false);
        self.engine
            .english_mode(EventState::MouseDown, 0, false, false);
        self.widths.clear();
    }

    pub fn metro(&self) -> bool {
        self.preferences.enabled(Setting::Metro)
            && self.app.eq_ignore_ascii_case("ApplicationFrameHost.exe")
    }

    pub fn foreground(&mut self, exe: &str) -> bool {
        use Setting::*;
        self.reset();
        if exe.eq_ignore_ascii_case("explorer.exe") {
            return false;
        }
        self.app.clear();
        if exe.is_empty() || exe.len() > 255 {
            return false;
        }
        self.app.push_str(exe);
        if !self.preferences.enabled(SmartSwitch) && !self.preferences.enabled(RememberTable) {
            return false;
        }
        let current = self.packed_app_settings();
        let at_capacity = self.smart_switch.iter().size_hint().0 >= u16::MAX as usize;
        let stored = if at_capacity {
            let Some((_, value)) = self.smart_switch.iter().find(|(name, _)| *name == exe) else {
                return false;
            };
            i32::from(value)
        } else {
            self.smart_switch.get(exe, current)
        };
        if stored < 0 {
            return true;
        }
        if self.preferences.enabled(SmartSwitch) {
            self.preferences.set(Language, stored as u32 & 1);
        }
        if self.preferences.enabled(RememberTable) && stored >> 1 < 5 {
            self.preferences.set(CodeTable, (stored >> 1) as u32);
        }
        self.refresh_settings();
        false
    }

    fn packed_app_settings(&self) -> i32 {
        SmartSwitch::packed(
            i64::from(self.preferences.get(Setting::Language)),
            i64::from(self.preferences.get(Setting::CodeTable)),
        )
    }

    pub fn remember_app(&mut self) -> bool {
        if self.app.is_empty()
            || (self.smart_switch.iter().size_hint().0 >= u16::MAX as usize
                && !self.smart_switch.iter().any(|(name, _)| name == self.app))
        {
            return false;
        }
        self.smart_switch.set(&self.app, self.packed_app_settings());
        true
    }

    fn mods(&self) -> u8 {
        [CTRL, ALT, WIN, SHIFT]
            .into_iter()
            .zip(self.modifiers.as_chunks::<2>().0)
            .filter(|(_, sides)| sides[0] || sides[1])
            .fold(0, |flags, (bit, _)| flags | bit)
    }

    fn shortcut(&self, vk: u16, mods: u8) -> Action {
        if Hotkey(self.preferences.get(Setting::SwitchKey)).matches(vk, mods) {
            Action::ToggleLanguage
        } else if Hotkey(self.preferences.get(Setting::ConvertKey)).matches(vk, mods) {
            Action::QuickConvert
        } else {
            Action::Pass
        }
    }

    pub fn key(&mut self, vk: u16, down: bool, caps_lock: bool, bypass: bool) -> Action {
        let previous = self.mods();
        if let Some(index) = [0xa2, 0xa3, 0xa4, 0xa5, 0x5b, 0x5c, 0xa0, 0xa1]
            .iter()
            .position(|&key| key == vk)
        {
            self.modifiers[index] = down;
            if down && previous == 0 {
                self.chord_used = bypass;
            }
            if !down && previous != self.mods() && !self.chord_used && !bypass {
                self.chord_used = true;
                let action = self.shortcut(u16::from(Hotkey::MODIFIERS_ONLY), previous);
                if action != Action::Pass {
                    return action;
                }
                if previous & CTRL != 0 && self.preferences.enabled(Setting::TemporarySpelling) {
                    self.engine.temp_off_spell_checking();
                }
                if previous & ALT != 0 && self.preferences.enabled(Setting::TemporaryEngine) {
                    self.engine.temp_off_engine(true);
                }
            }
            return Action::Pass;
        }
        let held = self.hotkey_down.get_mut(usize::from(vk));
        if let Some(held) = held {
            if *held {
                *held = down;
                return Action::Suppress;
            }
        }
        if !down {
            return Action::Pass;
        }
        self.chord_used |= previous != 0;
        if bypass || matches!(vk, 0x14 | 0x90 | 0x91 | 0xe7) {
            if bypass {
                self.reset();
            }
            return Action::Pass;
        }
        let action = self.shortcut(vk, previous);
        if action != Action::Pass {
            if let Some(held) = self.hotkey_down.get_mut(usize::from(vk)) {
                *held = true;
            }
            return action;
        }
        self.compose(vk, previous, caps_lock)
    }

    fn compose(&mut self, vk: u16, mods: u8, caps_lock: bool) -> Action {
        let key = to_engine(vk);
        let shift = mods & SHIFT != 0;
        // The Win32 hook treats only Alt and Ctrl as "other control" keys. The
        // Windows key remains a shortcut modifier and still reaches the target.
        let control = mods & (ALT | CTRL) != 0;
        let english = !self.preferences.enabled(Setting::Language);
        let table = self.preferences.get(Setting::CodeTable);
        let double = matches!(table, 2 | 3);
        let metro = self.metro();
        let hook = if english {
            if !self.preferences.enabled(Setting::UseMacro)
                || !self.preferences.enabled(Setting::EnglishMacro)
            {
                return Action::Pass;
            }
            self.engine
                .english_mode(EventState::KeyDown, key, caps_lock || shift, control)
        } else {
            let caps = match (shift, caps_lock) {
                (true, false) => 1,
                (false, true) => 2,
                _ => 0,
            };
            self.engine
                .handle_event(Event::Keyboard, EventState::KeyDown, key, caps, control)
        };
        let code = i64::from(hook.code);
        if code == vDoNothing || (english && code != vReplaceMaro) {
            if double {
                let state = if english {
                    if key == KEY_DELETE as u16 {
                        2
                    } else if !control && key_code_to_character(u32::from(key)) != 0 {
                        3
                    } else {
                        1
                    }
                } else {
                    hook.ext_code
                };
                match state {
                    1 => self.widths.clear(),
                    2 if self.widths.pop_back() == Some(2) => return Action::ExtraBackspace,
                    3 => push_width(&mut self.widths, 1),
                    _ => {}
                }
            }
            return Action::Pass;
        }
        if code == vBreakWord {
            self.reset();
            return Action::Suppress;
        }
        let is_macro = code == vReplaceMaro;
        let delete = usize::from(hook.backspace_count);
        let workaround = if !self.preferences.enabled(Setting::BrowserFix)
            || (!is_macro && hook.ext_code == 4)
        {
            Workaround::None
        } else if !is_macro
            && self.preferences.enabled(Setting::ChromiumFix)
            && ["chrome.exe", "brave.exe", "msedge.exe"]
                .iter()
                .any(|exe| self.app.eq_ignore_ascii_case(exe))
        {
            if delete == 0 {
                Workaround::None
            } else {
                Workaround::SelectLeft
            }
        } else {
            Workaround::Placeholder
        };

        let mut backspaces = (0..delete)
            .map(|_| usize::from(self.widths.pop_back().unwrap_or(1)))
            .sum();
        if workaround == Workaround::Placeholder {
            backspaces += 1;
        } else if workaround == Workaround::SelectLeft && delete == 1 {
            // The original decrements the logical count before expanding UTF-16 widths,
            // so one selected character replaces a compound unit without backspaces.
            backspaces = 0;
        }
        let mut characters: Vec<_> = if is_macro {
            hook.macro_data
                .iter()
                .copied()
                .map(|data| decode(data, table))
                .collect()
        } else {
            let count = usize::from(hook.new_char_count).min(hook.char_data.len());
            hook.char_data[..count]
                .iter()
                .rev()
                .copied()
                .map(|data| decode(data, table))
                .collect()
        };
        let mut macro_trigger = None;
        if is_macro || (code == vRestore || code == vRestoreAndStartNewSession) {
            let caps = if is_macro { shift } else { shift || caps_lock };
            let resolved =
                key_code_to_character(u32::from(key) | if caps { CAPS_MASK as u32 } else { 0 });
            let trigger = if resolved == 0 {
                Character::Key(vk)
            } else {
                Character::Text([resolved, 0], 1)
            };
            if is_macro {
                macro_trigger = Some(trigger);
            } else {
                characters.push(trigger);
            }
        }
        if double {
            for character in characters.iter().chain(macro_trigger.iter()) {
                if let Character::Text(_, width) = character {
                    push_width(&mut self.widths, *width);
                }
            }
        }
        if code == vRestoreAndStartNewSession {
            self.engine.start_new_session();
        }
        Action::Replace(Edit {
            backspaces,
            characters,
            macro_trigger,
            workaround,
            metro,
            step_by_step: self.preferences.enabled(Setting::StepByStep),
        })
    }
}

fn push_width(widths: &mut VecDeque<u8>, width: u8) {
    if widths.len() == 255 {
        widths.pop_front();
    }
    widths.push_back(width);
}

fn decode(packed: u32, table: u32) -> Character {
    let unit = packed as u16;
    let single = |unit| Character::Text([unit, 0], 1);
    if packed & PURE_CHARACTER_MASK as u32 != 0 {
        return single(unit);
    }
    if packed & CHAR_CODE_MASK as u32 == 0 {
        let character = key_code_to_character(packed);
        return if character != 0 {
            single(character)
        } else {
            Character::Key(
                KEYS.iter()
                    .find(|entry| entry.1 == i64::from(unit))
                    .map_or(0, |entry| entry.0),
            )
        };
    }
    match table {
        1 | 2 | 4 if unit >> 8 > 32 => Character::Text([unit & 0xff, unit >> 8], 2),
        1 | 2 | 4 => single(unit & 0xff),
        3 => match (unit >> 13)
            .checked_sub(1)
            .and_then(|index| _unicodeCompoundMark.get(index as usize))
        {
            Some(mark) => Character::Text([unit & 0x1fff, *mark as u16], 2),
            None => single(unit & 0x1fff),
        },
        _ => single(unit),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn windows_letter_and_vni_number_use_engine_codes() {
        assert_eq!(to_engine(0x41), KEY_A as u16);
        assert_eq!(to_engine(0x31), KEY_1 as u16);
        assert_eq!(to_engine(0x70), KEY_EMPTY as u16);
    }

    #[test]
    fn telex_edit_and_compound_backspace_follow_win32_widths() {
        let mut preferences = Preferences::default();
        preferences.set(Setting::CodeTable, 3);
        preferences.set(Setting::BrowserFix, 0);
        let mut typing = Typing::new(preferences);
        assert_eq!(typing.key(0x41, true, false, false), Action::Pass);
        assert_eq!(
            typing.key(0x53, true, false, false),
            Action::Replace(Edit {
                backspaces: 1,
                characters: vec![Character::Text([0x61, 0x301], 2)],
                macro_trigger: None,
                workaround: Workaround::None,
                step_by_step: true,
                metro: false,
            })
        );
        assert_eq!(typing.key(8, true, false, false), Action::ExtraBackspace);
    }

    #[test]
    fn home_ends_the_word_before_the_next_tone_key() {
        let mut preferences = Preferences::default();
        preferences.set(Setting::RestoreSpelling, 0);
        let mut typing = Typing::new(preferences);
        assert_eq!(typing.key(0x41, true, false, false), Action::Pass);
        assert_eq!(typing.key(0x24, true, false, false), Action::Pass);
        assert_eq!(typing.key(0x53, true, false, false), Action::Pass);
    }

    #[test]
    fn english_macros_do_not_carry_an_abbreviation_between_apps() {
        let mut preferences = Preferences::default();
        preferences.set(Setting::Language, 0);
        preferences.set(Setting::EnglishMacro, 1);
        preferences.set(Setting::BrowserFix, 0);
        preferences.set(Setting::StepByStep, 0);
        let mut typing = Typing::new(preferences);
        typing.engine.add_macro("vn", "😀");
        typing.foreground("first.exe");
        typing.key(0x56, true, false, false);
        typing.key(0x4e, true, false, false);
        typing.foreground("second.exe");
        assert_eq!(typing.key(0x20, true, false, false), Action::Pass);
        typing.key(0x56, true, false, false);
        typing.key(0x4e, true, false, false);
        assert_eq!(
            typing.key(0x20, true, false, false),
            Action::Replace(Edit {
                backspaces: 2,
                characters: vec![
                    Character::Text([0xd83d, 0], 1),
                    Character::Text([0xde00, 0], 1)
                ],
                macro_trigger: Some(Character::Text([0x20, 0], 1)),
                workaround: Workaround::None,
                step_by_step: false,
                metro: false,
            })
        );
    }

    #[test]
    fn modifier_only_shortcut_ignores_a_used_chord() {
        let mut preferences = Preferences::default();
        preferences.set(Setting::SwitchKey, 0xfe00_03fe);
        let mut typing = Typing::new(preferences);
        typing.key(0xa2, true, false, false);
        typing.key(0xa4, true, false, false);
        typing.key(0x58, true, false, false);
        assert_eq!(typing.key(0xa4, false, false, false), Action::Pass);
        typing.key(0xa2, false, false, false);
        typing.key(0xa2, true, false, false);
        typing.key(0xa4, true, false, false);
        assert_eq!(
            typing.key(0xa4, false, false, false),
            Action::ToggleLanguage
        );
        assert_eq!(typing.key(0xa2, false, false, false), Action::Pass);
    }

    #[test]
    fn windows_key_does_not_stop_the_following_tone() {
        let mut typing = Typing::new(Preferences::default());
        typing.key(0x5b, true, false, false);
        assert_eq!(typing.key(0x41, true, false, false), Action::Pass);
        typing.key(0x5b, false, false, false);
        assert_eq!(
            typing.key(0x53, true, false, false),
            Action::Replace(Edit {
                backspaces: 2,
                characters: vec![Character::Text([0x00e1, 0], 1)],
                macro_trigger: None,
                workaround: Workaround::Placeholder,
                step_by_step: true,
                metro: false,
            })
        );
    }

    #[test]
    fn control_still_keeps_the_letter_out_of_the_word() {
        let mut typing = Typing::new(Preferences::default());
        typing.key(0xa2, true, false, false);
        assert_eq!(typing.key(0x41, true, false, false), Action::Pass);
        typing.key(0xa2, false, false, false);
        assert_eq!(typing.key(0x53, true, false, false), Action::Pass);
    }

    #[test]
    fn chromium_selects_one_logical_character_without_backspacing_its_units() {
        let mut preferences = Preferences::default();
        preferences.set(Setting::CodeTable, 3);
        preferences.set(Setting::ChromiumFix, 1);
        let mut typing = Typing::new(preferences);
        typing.foreground("chrome.exe");
        assert_eq!(typing.key(0x41, true, false, false), Action::Pass);
        match typing.key(0x53, true, false, false) {
            Action::Replace(edit) => {
                assert_eq!(edit.workaround, Workaround::SelectLeft);
                assert_eq!(edit.backspaces, 0);
            }
            other => panic!("expected the first tone to select, got {other:?}"),
        }
        match typing.key(0x46, true, false, false) {
            Action::Replace(edit) => {
                assert_eq!(edit.workaround, Workaround::SelectLeft);
                assert_eq!(edit.backspaces, 0);
            }
            other => panic!("expected the tone change to select, got {other:?}"),
        }
    }

    #[test]
    fn language_switch_keeps_a_temporary_stop() {
        let mut preferences = Preferences::default();
        preferences.set(Setting::TemporaryEngine, 1);
        preferences.set(Setting::TemporarySpelling, 1);
        preferences.set(Setting::BrowserFix, 0);
        let mut typing = Typing::new(preferences);
        typing.key(0xa4, true, false, false);
        typing.key(0xa4, false, false, false);
        typing.key(0xa2, true, false, false);
        typing.key(0xa2, false, false, false);
        assert_eq!(typing.engine.settings().vCheckSpelling, 0);
        typing.preferences.set(Setting::Language, 0);
        typing.refresh_settings();
        typing.preferences.set(Setting::Language, 1);
        typing.refresh_settings();
        assert_eq!(typing.engine.settings().vCheckSpelling, 0);
        assert_eq!(typing.key(0x41, true, false, false), Action::Pass);
        assert_eq!(typing.key(0x53, true, false, false), Action::Pass);
    }

    #[test]
    fn switching_apps_ends_a_temporary_engine_stop() {
        let mut preferences = Preferences::default();
        preferences.set(Setting::TemporaryEngine, 1);
        preferences.set(Setting::BrowserFix, 0);
        let mut typing = Typing::new(preferences);
        typing.key(0xa4, true, false, false);
        assert_eq!(typing.key(0xa4, false, false, false), Action::Pass);
        assert_eq!(typing.key(0x41, true, false, false), Action::Pass);
        assert_eq!(typing.key(0x53, true, false, false), Action::Pass);
        typing.foreground("notepad.exe");
        assert_eq!(typing.key(0x41, true, false, false), Action::Pass);
        assert!(matches!(
            typing.key(0x53, true, false, false),
            Action::Replace(_)
        ));
    }

    #[test]
    fn hotkey_box_accepts_character_keys_and_clears_on_backspace_or_delete() {
        assert_eq!(capture_key(0x5a), Some(0x5a));
        assert_eq!(capture_key(0x20), Some(0x20));
        assert_eq!(capture_key(0xba), Some(0xba));
        assert_eq!(capture_key(0x08), Some(Hotkey::MODIFIERS_ONLY));
        assert_eq!(capture_key(0x2e), Some(Hotkey::MODIFIERS_ONLY));
        assert_eq!(capture_key(0x70), None);
        assert_eq!(capture_key(0x25), None);
        assert_eq!(capture_key(0x1b), None);
    }

    #[test]
    fn smart_switch_restores_language_and_table_independently() {
        let mut typing = Typing::new(Preferences::default());
        assert!(typing.foreground("one.exe"));
        typing.preferences.set(Setting::CodeTable, 2);
        typing.preferences.set(Setting::Language, 0);
        assert!(typing.remember_app());
        assert!(typing.foreground("two.exe"));
        typing.preferences.set(Setting::CodeTable, 3);
        typing.preferences.set(Setting::Language, 1);
        assert!(!typing.foreground("one.exe"));
        assert_eq!(typing.engine.settings().vCodeTable, 2);
        assert_eq!(typing.engine.settings().vLanguage, 0);
    }
}
