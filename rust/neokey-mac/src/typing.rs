use std::collections::VecDeque;

use neokey_engine::data::*;
use neokey_engine::vietnamese::{_unicodeCompoundMark, key_code_to_character};
use neokey_engine::{Engine, Event, EventState, Settings};

// Core Graphics flag values. Keeping this layer independent of Core Graphics
// lets it run with recorded inputs or in a benchmark without an event tap.
const CAPS_LOCK: u64 = 1 << 16;
const SHIFT: u64 = 1 << 17;
const CONTROL: u64 = 1 << 18;
const OPTION: u64 = 1 << 19;
const COMMAND: u64 = 1 << 20;
const SHORTCUT_MODIFIERS: u64 = SHIFT | CONTROL | OPTION | COMMAND;
const OTHER_CONTROL: u64 = CONTROL | OPTION | COMMAND | (0b111 << 21);
const MODIFIERS_ONLY: u16 = 0xfe;
const EMPTY_HOTKEY: u32 = 0xfe00_00fe;
pub const DEFAULT_HOTKEY: u32 = 0x7a00_0206;

#[derive(Clone, Copy)]
pub enum Input {
    KeyDown {
        key: u16,
        layout_key: u16,
        flags: u64,
        repeat: bool,
    },
    KeyUp(u16),
    FlagsChanged(u64),
    MouseDown,
}

#[derive(Debug, PartialEq, Eq)]
pub enum OutputAction {
    Pass,
    Suppress,
    ToggleLanguage,
    Replace(Edit),
    ExtraBackspaces(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub enum LauncherTarget<T> {
    Regular,
    SelectOnly,
    Focused(T),
}

impl OutputAction {
    pub fn focused_launcher<T>(
        &self,
        event_launcher: bool,
        lookup: impl FnOnce() -> Option<(AppBehavior, T)>,
    ) -> LauncherTarget<T> {
        let deletes = match self {
            Self::Replace(edit) => edit.backspaces,
            Self::ExtraBackspaces(count) => *count,
            _ => 0,
        };
        if deletes == 0 {
            return LauncherTarget::Regular;
        }
        match lookup() {
            Some((owner, element)) if owner.launcher => LauncherTarget::Focused(element),
            None if event_launcher => LauncherTarget::SelectOnly,
            _ => LauncherTarget::Regular,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Edit {
    pub backspaces: usize,
    pub launcher_selection: usize,
    pub utf16_delete: usize,
    pub characters: Vec<Character>,
    pub replay_trigger: bool,
    pub workaround: Workaround,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Workaround {
    None,
    Placeholder(u16),
    SelectBeforeDelete { selection: usize, backspaces: usize },
}

#[derive(Debug, PartialEq, Eq)]
pub enum Character {
    Text { units: [u16; 2], len: usize },
    Key { code: u16, shift: bool },
}

#[derive(Clone, Copy, Default)]
pub struct AppBehavior {
    pub launcher: bool,
    pub compound_graphemes: bool,
    pub chromium: bool,
    pub zero_width_placeholder: bool,
}

impl AppBehavior {
    pub fn for_bundle(id: &str) -> Self {
        let launcher = [
            "com.apple.Spotlight",
            "com.apple.campo",
            "com.runningwithcrayons.Alfred",
            "com.raycast.macos",
        ]
        .iter()
        .any(|prefix| {
            id == *prefix
                || id
                    .strip_prefix(prefix)
                    .is_some_and(|suffix| suffix.starts_with('.'))
        }) || matches!(
            id,
            "com.runningwithcrayons.Alfred-3" | "Spotlight" | "Alfred" | "Raycast"
        );
        let chromium = [
            "com.google.Chrome",
            "com.brave.Browser",
            "com.microsoft.edgemac.Dev",
            "com.microsoft.edgemac.Beta",
            "com.microsoft.Edge.Dev",
            "com.microsoft.Edge",
        ]
        .iter()
        .any(|prefix| id.starts_with(prefix));
        Self {
            launcher,
            compound_graphemes: launcher || chromium || id.starts_with("com.apple."),
            chromium,
            zero_width_placeholder: matches!(id, "com.sublimetext.2" | "com.sublimetext.3"),
        }
    }
}

pub struct Typing {
    engine: Engine,
    app: AppBehavior,
    chromium_fix: bool,
    blocked_layout: bool,
    widths: VecDeque<usize>,
    // macOS virtual key codes are seven bits.
    consumed_keys: [bool; 128],
    shortcut_down: Option<u16>,
    held_modifiers: u64,
    modifiers_used: bool,
}

impl Typing {
    pub fn new(settings: Settings) -> Self {
        Self {
            engine: Engine::with_settings(settings),
            app: AppBehavior::default(),
            chromium_fix: false,
            blocked_layout: false,
            widths: VecDeque::new(),
            consumed_keys: [false; 128],
            shortcut_down: None,
            held_modifiers: 0,
            modifiers_used: false,
        }
    }

    pub fn engine(&self) -> &Engine {
        &self.engine
    }

    pub fn engine_mut(&mut self) -> &mut Engine {
        &mut self.engine
    }

    pub fn set_app(&mut self, app: AppBehavior) {
        self.app = app;
        self.reset_session();
    }

    pub fn set_chromium_fix(&mut self, enabled: bool) {
        self.chromium_fix = enabled;
    }

    pub fn set_blocked_layout(&mut self, blocked: bool) {
        if self.blocked_layout != blocked {
            self.blocked_layout = blocked;
            self.reset_session();
        }
    }

    pub fn reset_session(&mut self) {
        self.engine
            .handle_event(Event::Mouse, EventState::MouseDown, 0, 0, false);
        self.engine
            .english_mode(EventState::MouseDown, 0, false, false);
        self.widths.clear();
    }

    pub fn reset_after_tap_disable(&mut self) {
        self.reset_session();
        self.consumed_keys.fill(false);
        self.shortcut_down = None;
        self.held_modifiers = 0;
        self.modifiers_used = false;
    }

    pub fn process_key(&mut self, input: Input) -> OutputAction {
        match input {
            Input::MouseDown => {
                self.reset_session();
                self.modifiers_used = self.held_modifiers != 0;
                OutputAction::Pass
            }
            Input::FlagsChanged(flags) => self.flags_changed(flags & SHORTCUT_MODIFIERS),
            Input::KeyUp(key) => {
                if self.shortcut_down == Some(key) {
                    self.shortcut_down = None;
                }
                match self.consumed_keys.get_mut(key as usize) {
                    Some(consumed) if *consumed => {
                        *consumed = false;
                        OutputAction::Suppress
                    }
                    _ => OutputAction::Pass,
                }
            }
            Input::KeyDown {
                key,
                layout_key,
                flags,
                repeat,
            } => {
                let action = self.key_down(key, layout_key, flags, repeat);
                if let Some(consumed) = self.consumed_keys.get_mut(key as usize) {
                    let passes_down = match &action {
                        OutputAction::Pass | OutputAction::ExtraBackspaces(_) => true,
                        OutputAction::Replace(edit) => edit.replay_trigger,
                        OutputAction::Suppress | OutputAction::ToggleLanguage => false,
                    };
                    // One physical key-up follows the initial key-down and any repeats.
                    // Pass it if the foreground app received any of those key-downs.
                    if passes_down {
                        *consumed = false;
                    } else if !repeat {
                        *consumed = true;
                    }
                }
                action
            }
        }
    }

    fn hotkey_matches(&self, key: u16, flags: u64) -> bool {
        let setting = self.engine.settings().vSwitchKeyStatus as u32;
        let modifiers = (((setting >> 8) & 1) as u64 * CONTROL)
            | (((setting >> 9) & 1) as u64 * OPTION)
            | (((setting >> 10) & 1) as u64 * COMMAND)
            | (((setting >> 11) & 1) as u64 * SHIFT);
        setting != 0
            && setting & !0x8000 != EMPTY_HOTKEY
            && key == (setting & 0xff) as u16
            && flags & SHORTCUT_MODIFIERS == modifiers
    }

    fn flags_changed(&mut self, modifiers: u64) -> OutputAction {
        let previous = std::mem::replace(&mut self.held_modifiers, modifiers);
        if previous == 0 {
            self.modifiers_used = false;
        }
        if previous & !modifiers == 0 || self.modifiers_used {
            return OutputAction::Pass;
        }
        if self.hotkey_matches(MODIFIERS_ONLY, previous) {
            self.modifiers_used = true;
            return OutputAction::ToggleLanguage;
        }
        if self.engine.settings().vTempOffSpelling != 0 && previous & CONTROL != 0 {
            self.engine.temp_off_spell_checking();
        }
        if self.engine.settings().vTempOffOpenKey != 0 && previous & COMMAND != 0 {
            self.engine.temp_off_engine(true);
        }
        self.modifiers_used = true;
        OutputAction::Pass
    }

    fn key_down(&mut self, physical_key: u16, key: u16, flags: u64, repeat: bool) -> OutputAction {
        self.modifiers_used |= flags & SHORTCUT_MODIFIERS != 0;
        if repeat && self.shortcut_down == Some(physical_key) {
            return OutputAction::Suppress;
        }
        if !repeat && self.hotkey_matches(key, flags) {
            self.shortcut_down = Some(physical_key);
            return OutputAction::ToggleLanguage;
        }

        let caps = if flags & SHIFT != 0 {
            1
        } else if flags & CAPS_LOCK != 0 {
            2
        } else {
            0
        };
        let control = flags & OTHER_CONTROL != 0;
        let settings = self.engine.settings();
        let table = settings.vCodeTable;
        let recommend = settings.vFixRecommendBrowser != 0 && !self.app.launcher;
        let english = settings.vLanguage == 0;
        let hook = if english {
            if settings.vUseMacro == 0 || settings.vUseMacroInEnglishMode == 0 {
                return OutputAction::Pass;
            }
            self.engine
                .english_mode(EventState::KeyDown, key, caps != 0, control)
        } else {
            if self.blocked_layout {
                return OutputAction::Pass;
            }
            self.engine
                .handle_event(Event::Keyboard, EventState::KeyDown, key, caps, control)
        };
        let code = i64::from(hook.code);
        let double = matches!(table, 2 | 3);
        let separate_delete = table == 2 || !self.app.compound_graphemes;
        if code == vDoNothing {
            if english || !double {
                return OutputAction::Pass;
            }
            return match hook.ext_code {
                1 => {
                    self.widths.clear();
                    OutputAction::Pass
                }
                2 if self.widths.pop_back() == Some(2) && separate_delete => {
                    OutputAction::ExtraBackspaces(1)
                }
                3 => {
                    self.push_width(1);
                    OutputAction::Pass
                }
                _ => OutputAction::Pass,
            };
        }

        let macro_output = code == vReplaceMaro;
        let replay_trigger = macro_output || code == vRestore || code == vRestoreAndStartNewSession;
        if code == vBreakWord {
            self.reset_session();
            return OutputAction::Suppress;
        }

        let delete = usize::from(hook.backspace_count);
        let mut workaround = if !recommend || (hook.ext_code == 4 && !macro_output) {
            Workaround::None
        } else if self.chromium_fix && self.app.chromium && !macro_output {
            if delete == 0 {
                Workaround::None
            } else {
                let width = self.widths.back().copied().unwrap_or(1);
                let selection = if separate_delete { width } else { 1 };
                Workaround::SelectBeforeDelete {
                    selection,
                    backspaces: 0,
                }
            }
        } else {
            Workaround::Placeholder(if self.app.zero_width_placeholder {
                0x200c
            } else {
                0x202f
            })
        };
        let characters = if macro_output {
            decode_macro(&hook.macro_data, table)
        } else {
            let count = usize::from(hook.new_char_count).min(hook.char_data.len());
            hook.char_data[..count]
                .iter()
                .rev()
                .map(|&unit| decode(unit, table))
                .collect()
        };

        let mut utf16_delete = 0;
        let mut backspaces = 0;
        for _ in 0..delete {
            let width = self.widths.pop_back().unwrap_or(1);
            utf16_delete += width;
            backspaces += if separate_delete { width } else { 1 };
        }
        if let Workaround::SelectBeforeDelete {
            selection,
            backspaces: after_selection,
        } = &mut workaround
        {
            if delete > 1 || (characters.is_empty() && !replay_trigger) {
                // The first Backspace erases the selection as a whole.
                *after_selection = backspaces - *selection + 1;
            }
        }
        if double && !english {
            for character in &characters {
                self.push_width(match character {
                    Character::Text { len, .. } => *len,
                    Character::Key { .. } => 1,
                });
            }
            if replay_trigger && key_code_to_character(u32::from(key)) != 0 {
                self.push_width(1);
            }
        }
        if code == vRestoreAndStartNewSession {
            self.engine.start_new_session();
        }
        OutputAction::Replace(Edit {
            backspaces,
            launcher_selection: if table == 2 { backspaces } else { delete },
            utf16_delete,
            characters,
            replay_trigger,
            workaround,
        })
    }

    fn push_width(&mut self, width: usize) {
        // The hook can request at most 255 logical deletions.
        if self.widths.len() == 255 {
            self.widths.pop_front();
        }
        self.widths.push_back(width);
    }
}

fn decode(packed: u32, table: i64) -> Character {
    let unit = packed as u16;
    let single = |unit| Character::Text {
        units: [unit, 0],
        len: 1,
    };
    if packed & PURE_CHARACTER_MASK as u32 != 0 {
        return single(unit);
    }
    if packed & CHAR_CODE_MASK as u32 == 0 {
        let character = key_code_to_character(packed);
        return if character == 0 {
            Character::Key {
                code: unit,
                shift: packed & CAPS_MASK as u32 != 0,
            }
        } else {
            single(character)
        };
    }
    let (base, mark) = match table {
        1 | 2 | 4 => (unit & 0xff, (unit >> 8 > 32).then_some(unit >> 8)),
        3 => (
            unit & 0x1fff,
            (unit >> 13)
                .checked_sub(1)
                .and_then(|index| _unicodeCompoundMark.get(index as usize))
                .map(|mark| *mark as u16),
        ),
        _ => (unit, None),
    };
    match mark {
        Some(mark) => Character::Text {
            units: [base, mark],
            len: 2,
        },
        None => single(base),
    }
}

fn decode_macro(data: &[u32], table: i64) -> Vec<Character> {
    let mut decoded = Vec::with_capacity(data.len());
    let mut remaining = data;
    while let Some((&unit, rest)) = remaining.split_first() {
        // Keep a supplementary Unicode character intact even in one-by-one mode.
        let high = unit as u16;
        if unit & PURE_CHARACTER_MASK as u32 != 0 && (0xd800..=0xdbff).contains(&high) {
            if let Some((&next, tail)) = rest.split_first() {
                let low = next as u16;
                if next & PURE_CHARACTER_MASK as u32 != 0 && (0xdc00..=0xdfff).contains(&low) {
                    decoded.push(Character::Text {
                        units: [high, low],
                        len: 2,
                    });
                    remaining = tail;
                    continue;
                }
            }
        }
        decoded.push(decode(unit, table));
        remaining = rest;
    }
    decoded
}

#[cfg(test)]
mod tests {
    use super::*;

    fn down(key: i64, flags: u64) -> Input {
        Input::KeyDown {
            key: key as u16,
            layout_key: key as u16,
            flags,
            repeat: false,
        }
    }

    #[test]
    fn campo_is_a_launcher() {
        assert!(AppBehavior::for_bundle("com.apple.campo").launcher);
    }

    #[test]
    fn telex_edit_uses_the_focused_launcher_even_with_a_different_event_target() {
        let mut typing = Typing::new(Settings::default());
        let event_app = AppBehavior::for_bundle("com.apple.Terminal");
        assert!(!event_app.launcher);
        typing.set_app(event_app);

        let first = typing.process_key(down(KEY_O, 0));
        assert_eq!(first, OutputAction::Pass);
        assert_eq!(
            first.focused_launcher::<()>(event_app.launcher, || panic!(
                "pass-through key queried AX"
            )),
            LauncherTarget::Regular
        );
        let second = typing.process_key(down(KEY_O, 0));
        assert!(matches!(&second, OutputAction::Replace(edit) if edit.backspaces == 1));
        assert_eq!(
            second.focused_launcher(event_app.launcher, || Some((
                AppBehavior::for_bundle("com.apple.campo"),
                42
            ))),
            LauncherTarget::Focused(42)
        );
        assert_eq!(
            second.focused_launcher(true, || Some((event_app, 42))),
            LauncherTarget::Regular
        );
        assert_eq!(
            second.focused_launcher::<()>(true, || None),
            LauncherTarget::SelectOnly
        );
    }

    #[test]
    fn launcher_lookup_runs_only_for_actions_that_delete_text() {
        let insertion = OutputAction::Replace(Edit {
            backspaces: 0,
            launcher_selection: 0,
            utf16_delete: 0,
            characters: vec![],
            replay_trigger: false,
            workaround: Workaround::Placeholder(0x202f),
        });
        for action in [
            insertion,
            OutputAction::Suppress,
            OutputAction::ToggleLanguage,
            OutputAction::ExtraBackspaces(0),
        ] {
            assert_eq!(
                action
                    .focused_launcher::<()>(true, || panic!("action without deletion queried AX")),
                LauncherTarget::Regular
            );
        }
        assert_eq!(
            OutputAction::ExtraBackspaces(1).focused_launcher(false, || Some((
                AppBehavior::for_bundle("com.apple.campo"),
                42
            ))),
            LauncherTarget::Focused(42)
        );
    }

    #[test]
    fn telex_replaces_the_previous_vowel_and_consumes_the_tone_key() {
        let mut typing = Typing::new(Settings::default());
        assert_eq!(typing.process_key(down(KEY_A, 0)), OutputAction::Pass);
        assert_eq!(
            typing.process_key(down(KEY_S, 0)),
            OutputAction::Replace(Edit {
                backspaces: 1,
                launcher_selection: 1,
                utf16_delete: 1,
                characters: vec![Character::Text {
                    units: [0x00e1, 0],
                    len: 1
                }],
                replay_trigger: false,
                workaround: Workaround::None,
            })
        );
        assert_eq!(
            typing.process_key(Input::KeyUp(KEY_S as u16)),
            OutputAction::Suppress
        );
    }

    #[test]
    fn mouse_down_ends_the_current_composition() {
        let mut typing = Typing::new(Settings::default());
        typing.process_key(down(KEY_A, 0));
        assert_eq!(typing.process_key(Input::MouseDown), OutputAction::Pass);
        assert_eq!(typing.process_key(down(KEY_S, 0)), OutputAction::Pass);
    }

    #[test]
    fn option_z_requests_a_language_switch_and_consumes_the_key() {
        let mut typing = Typing::new(Settings {
            vSwitchKeyStatus: i64::from(DEFAULT_HOTKEY),
            ..Settings::default()
        });
        assert_eq!(
            typing.process_key(down(KEY_Z, 1 << 19)),
            OutputAction::ToggleLanguage
        );
        assert_eq!(
            typing.process_key(Input::KeyUp(KEY_Z as u16)),
            OutputAction::Suppress
        );
    }
    #[test]
    fn repeated_tone_restores_the_vowel_and_keeps_the_trigger() {
        let mut typing = Typing::new(Settings::default());
        typing.process_key(down(KEY_A, 0));
        typing.process_key(down(KEY_S, 0));
        typing.process_key(Input::KeyUp(KEY_S as u16));
        assert_eq!(
            typing.process_key(down(KEY_S, 0)),
            OutputAction::Replace(Edit {
                backspaces: 1,
                launcher_selection: 1,
                utf16_delete: 1,
                characters: vec![Character::Text {
                    units: [b'a' as u16, 0],
                    len: 1
                }],
                replay_trigger: true,
                workaround: Workaround::None,
            })
        );
        assert_eq!(
            typing.process_key(Input::KeyUp(KEY_S as u16)),
            OutputAction::Pass
        );
    }

    #[test]
    fn compound_launcher_edit_counts_utf16_units_and_selects_one_grapheme() {
        let mut typing = Typing::new(Settings {
            vCodeTable: 3,
            vFixRecommendBrowser: 1,
            ..Settings::default()
        });
        typing.set_app(AppBehavior::for_bundle("com.runningwithcrayons.Alfred-3"));
        typing.process_key(down(KEY_A, 0));
        typing.process_key(down(KEY_S, 0));
        let OutputAction::Replace(edit) = typing.process_key(down(KEY_F, 0)) else {
            panic!("changing the tone should replace the composed vowel");
        };
        assert_eq!(edit.utf16_delete, 2);
        assert_eq!(edit.backspaces, 1);
        assert_eq!(edit.launcher_selection, 1);
        assert_eq!(edit.workaround, Workaround::None);
        assert_eq!(
            edit.characters,
            vec![Character::Text {
                units: [b'a' as u16, 0x0300],
                len: 2
            }]
        );
    }

    #[test]
    fn compound_launcher_fallback_selects_one_grapheme_with_a_different_event_target() {
        let mut typing = Typing::new(Settings {
            vCodeTable: 3,
            ..Settings::default()
        });
        typing.set_app(AppBehavior::for_bundle("org.mozilla.firefox"));
        typing.process_key(down(KEY_A, 0));
        typing.process_key(down(KEY_S, 0));
        let OutputAction::Replace(edit) = typing.process_key(down(KEY_F, 0)) else {
            panic!("changing the tone should replace the composed vowel");
        };
        assert_eq!(edit.backspaces, 2);
        assert_eq!(edit.launcher_selection, 1);
    }

    #[test]
    fn english_macro_keeps_a_surrogate_pair_and_passes_the_space() {
        let mut typing = Typing::new(Settings {
            vLanguage: 0,
            vUseMacro: 1,
            vUseMacroInEnglishMode: 1,
            ..Settings::default()
        });
        assert!(typing.engine_mut().add_macro("vn", "😀"));
        typing.process_key(down(KEY_V, 0));
        typing.process_key(down(KEY_N, 0));
        assert_eq!(
            typing.process_key(down(KEY_SPACE, 0)),
            OutputAction::Replace(Edit {
                backspaces: 2,
                launcher_selection: 2,
                utf16_delete: 2,
                characters: vec![Character::Text {
                    units: [0xd83d, 0xde00],
                    len: 2
                }],
                replay_trigger: true,
                workaround: Workaround::None,
            })
        );
    }

    #[test]
    fn modifier_only_hotkey_fires_on_release_and_ignores_shortcut_chords() {
        let mut typing = Typing::new(Settings {
            vSwitchKeyStatus: 0xfe00_06fe,
            ..Settings::default()
        });
        typing.process_key(Input::FlagsChanged(COMMAND));
        typing.process_key(Input::FlagsChanged(COMMAND | OPTION));
        assert_eq!(
            typing.process_key(Input::FlagsChanged(OPTION)),
            OutputAction::ToggleLanguage
        );
        assert_eq!(
            typing.process_key(Input::FlagsChanged(0)),
            OutputAction::Pass
        );

        typing.process_key(Input::FlagsChanged(COMMAND | OPTION));
        typing.process_key(down(KEY_X, COMMAND | OPTION));
        assert_eq!(
            typing.process_key(Input::FlagsChanged(0)),
            OutputAction::Pass
        );
    }

    #[test]
    fn releasing_command_temporarily_bypasses_composition() {
        let mut typing = Typing::new(Settings {
            vTempOffOpenKey: 1,
            ..Settings::default()
        });
        typing.process_key(Input::FlagsChanged(COMMAND));
        typing.process_key(Input::FlagsChanged(0));
        assert_eq!(typing.process_key(down(KEY_A, 0)), OutputAction::Pass);
        assert_eq!(typing.process_key(down(KEY_S, 0)), OutputAction::Pass);
        typing.process_key(down(KEY_SPACE, 0));
        typing.process_key(down(KEY_A, 0));
        assert!(matches!(
            typing.process_key(down(KEY_S, 0)),
            OutputAction::Replace(_)
        ));
    }
    #[test]
    fn passes_key_up_if_a_tone_repeat_replayed_the_physical_key_down() {
        let mut typing = Typing::new(Settings::default());
        typing.process_key(down(KEY_A, 0));
        typing.process_key(down(KEY_S, 0));
        let repeated = Input::KeyDown {
            key: KEY_S as u16,
            layout_key: KEY_S as u16,
            flags: 0,
            repeat: true,
        };
        assert!(matches!(
            typing.process_key(repeated),
            OutputAction::Replace(Edit {
                replay_trigger: true,
                ..
            })
        ));
        assert_eq!(
            typing.process_key(Input::KeyUp(KEY_S as u16)),
            OutputAction::Pass
        );
    }
    #[test]
    fn chromium_autocomplete_fix_selects_before_deleting_multiple_letters() {
        let mut typing = Typing::new(Settings {
            vFixRecommendBrowser: 1,
            ..Settings::default()
        });
        typing.set_app(AppBehavior::for_bundle("com.google.Chrome"));
        typing.set_chromium_fix(true);
        for key in [KEY_H, KEY_O, KEY_A] {
            typing.process_key(down(key, 0));
        }
        let OutputAction::Replace(edit) = typing.process_key(down(KEY_F, 0)) else {
            panic!("adding the tone should replace two letters");
        };
        assert_eq!(edit.backspaces, 2);
        assert_eq!(
            edit.workaround,
            Workaround::SelectBeforeDelete {
                selection: 1,
                backspaces: 2
            }
        );
    }
    #[test]
    fn vni_windows_accounts_for_both_units_when_the_user_deletes_a_vowel() {
        let mut typing = Typing::new(Settings {
            vInputType: vVNI,
            vCodeTable: 2,
            ..Settings::default()
        });
        typing.process_key(down(KEY_A, 0));
        let OutputAction::Replace(edit) = typing.process_key(down(KEY_1, 0)) else {
            panic!("VNI tone key should replace the preceding vowel");
        };
        assert!(matches!(
            edit.characters.as_slice(),
            [Character::Text { len: 2, .. }]
        ));
        assert_eq!(
            typing.process_key(down(KEY_DELETE, 0)),
            OutputAction::ExtraBackspaces(1)
        );
    }
}
