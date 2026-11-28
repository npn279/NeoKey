//! Encode and display `SwitchKeyStatus` / `vSwitchKeyStatus` values.
//!
//! Layout matches OpenKey and `Typing::hotkey_matches`:
//! - bits 0–7: key code (`0xFE` = modifiers only)
//! - bit 8: Control, 9: Option, 10: Command, 11: Shift
//! - bit 15 (`0x8000`): beep on language switch
//! - bits 24–31: display byte (OpenKey stored a character here)

use neokey_engine::data::{
    KEY_0, KEY_1, KEY_2, KEY_3, KEY_4, KEY_5, KEY_6, KEY_7, KEY_8, KEY_9, KEY_A, KEY_B, KEY_C,
    KEY_D, KEY_DELETE, KEY_E, KEY_ENTER, KEY_ESC, KEY_F, KEY_G, KEY_H, KEY_I, KEY_J, KEY_K, KEY_L,
    KEY_M, KEY_N, KEY_O, KEY_P, KEY_Q, KEY_R, KEY_RETURN, KEY_S, KEY_SPACE, KEY_T, KEY_TAB, KEY_U,
    KEY_V, KEY_W, KEY_X, KEY_Y, KEY_Z,
};
use neokey_mac::typing::DEFAULT_HOTKEY;

const MODIFIERS_ONLY: u32 = 0xfe;
const BEEP_BIT: u32 = 0x8000;
const CONTROL_BIT: u32 = 1 << 8;
const OPTION_BIT: u32 = 1 << 9;
const COMMAND_BIT: u32 = 1 << 10;
const SHIFT_BIT: u32 = 1 << 11;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hotkey {
    pub key: u8,
    pub control: bool,
    pub option: bool,
    pub command: bool,
    pub shift: bool,
    pub beep: bool,
}

impl Hotkey {
    pub fn from_status(status: i64) -> Self {
        let status = if status == 0 {
            DEFAULT_HOTKEY
        } else {
            status as u32
        };
        Self {
            key: (status & 0xff) as u8,
            control: status & CONTROL_BIT != 0,
            option: status & OPTION_BIT != 0,
            command: status & COMMAND_BIT != 0,
            shift: status & SHIFT_BIT != 0,
            beep: status & BEEP_BIT != 0,
        }
    }

    pub fn from_convert_status(status: i64) -> Self {
        Self::from_status(if status == 0 { 0xfe00_00fe } else { status }).with_beep(false)
    }

    pub fn to_status(self) -> i64 {
        let key = u32::from(self.key);
        let mut status = key | (key << 24);
        if self.control {
            status |= CONTROL_BIT;
        }
        if self.option {
            status |= OPTION_BIT;
        }
        if self.command {
            status |= COMMAND_BIT;
        }
        if self.shift {
            status |= SHIFT_BIT;
        }
        if self.beep {
            status |= BEEP_BIT;
        }
        status as i64
    }

    pub fn with_beep(mut self, beep: bool) -> Self {
        self.beep = beep;
        self
    }

    /// Compact symbols for the menu-bar panel hint, e.g. `⌥Z` or `⌃⇧`.
    pub fn format_symbols(&self) -> String {
        let mut out = String::new();
        if self.control {
            out.push('⌃');
        }
        if self.option {
            out.push('⌥');
        }
        if self.command {
            out.push('⌘');
        }
        if self.shift {
            out.push('⇧');
        }
        if self.key != MODIFIERS_ONLY as u8 {
            out.push_str(&key_label(self.key));
        }
        if out.is_empty() {
            out.push('—');
        }
        out
    }

    pub fn quick_convert_title(&self) -> String {
        let chord = self.format_symbols();
        if chord == "—" {
            "Chuyển mã nhanh".into()
        } else {
            format!("Chuyển mã nhanh - [{chord}]")
        }
    }

    /// Spaced labels for the settings capture control, e.g. `⌥ Option + Z`.
    pub fn format_labels(&self) -> String {
        let mut parts = Vec::new();
        if self.control {
            parts.push("⌃ Control".to_string());
        }
        if self.option {
            parts.push("⌥ Option".to_string());
        }
        if self.command {
            parts.push("⌘ Command".to_string());
        }
        if self.shift {
            parts.push("⇧ Shift".to_string());
        }
        if self.key != MODIFIERS_ONLY as u8 {
            parts.push(key_label(self.key));
        }
        if parts.is_empty() {
            "—".into()
        } else {
            parts.join(" + ")
        }
    }
}

/// Quartz modifier bits (`CGEventFlags`, same values as `NSEventModifierFlags`).
const FLAG_SHIFT: u64 = 1 << 17;
const FLAG_CONTROL: u64 = 1 << 18;
const FLAG_OPTION: u64 = 1 << 19;
const FLAG_COMMAND: u64 = 1 << 20;
const FLAG_MODIFIERS: u64 = FLAG_SHIFT | FLAG_CONTROL | FLAG_OPTION | FLAG_COMMAND;
const KEY_CODE_ESCAPE: u16 = 53;

/// What one event did to a running hotkey capture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capture {
    Pending,
    Done(Hotkey),
    Cancel,
}

/// Records the language-switch chord from raw event-tap events.
///
/// Modifier-only chords keep every modifier pressed during the chord, so
/// releasing ⌃ before ⇧ still records ⌃⇧ (not just the last one held).
/// A key chord needs ⌃, ⌥ or ⌘: a bare or Shift-only letter would toggle the
/// language while typing. Esc cancels.
#[derive(Clone, Copy, Debug)]
pub struct HotkeyRecorder {
    seen: u64,
    beep: bool,
}

impl HotkeyRecorder {
    pub fn new(beep: bool) -> Self {
        Self { seen: 0, beep }
    }

    pub fn flags_changed(&mut self, flags: u64) -> Capture {
        let held = flags & FLAG_MODIFIERS;
        if held != 0 {
            self.seen |= held;
            return Capture::Pending;
        }
        if self.seen == 0 {
            return Capture::Pending;
        }
        Capture::Done(self.hotkey(MODIFIERS_ONLY as u8, self.seen))
    }

    pub fn key_down(&mut self, key: u16, flags: u64) -> Capture {
        let held = flags & FLAG_MODIFIERS;
        if key == KEY_CODE_ESCAPE && held == 0 {
            return Capture::Cancel;
        }
        if held & !FLAG_SHIFT == 0 || key > 0xff || key == MODIFIERS_ONLY as u16 {
            // Wait for a real chord; a keystroke also ends any modifier-only try.
            self.seen = 0;
            return Capture::Pending;
        }
        Capture::Done(self.hotkey(key as u8, held))
    }

    fn hotkey(&self, key: u8, mods: u64) -> Hotkey {
        Hotkey {
            key,
            control: mods & FLAG_CONTROL != 0,
            option: mods & FLAG_OPTION != 0,
            command: mods & FLAG_COMMAND != 0,
            shift: mods & FLAG_SHIFT != 0,
            beep: self.beep,
        }
    }
}

fn key_label(code: u8) -> String {
    let code = i64::from(code);
    let letter = [
        (KEY_A, "A"),
        (KEY_B, "B"),
        (KEY_C, "C"),
        (KEY_D, "D"),
        (KEY_E, "E"),
        (KEY_F, "F"),
        (KEY_G, "G"),
        (KEY_H, "H"),
        (KEY_I, "I"),
        (KEY_J, "J"),
        (KEY_K, "K"),
        (KEY_L, "L"),
        (KEY_M, "M"),
        (KEY_N, "N"),
        (KEY_O, "O"),
        (KEY_P, "P"),
        (KEY_Q, "Q"),
        (KEY_R, "R"),
        (KEY_S, "S"),
        (KEY_T, "T"),
        (KEY_U, "U"),
        (KEY_V, "V"),
        (KEY_W, "W"),
        (KEY_X, "X"),
        (KEY_Y, "Y"),
        (KEY_Z, "Z"),
        (KEY_0, "0"),
        (KEY_1, "1"),
        (KEY_2, "2"),
        (KEY_3, "3"),
        (KEY_4, "4"),
        (KEY_5, "5"),
        (KEY_6, "6"),
        (KEY_7, "7"),
        (KEY_8, "8"),
        (KEY_9, "9"),
        (KEY_SPACE, "Space"),
        (KEY_TAB, "Tab"),
        (KEY_RETURN, "Return"),
        (KEY_ENTER, "Enter"),
        (KEY_ESC, "Esc"),
        (KEY_DELETE, "Delete"),
    ]
    .iter()
    .find(|(k, _)| *k == code)
    .map(|(_, label)| (*label).to_string());
    letter.unwrap_or_else(|| format!("0x{code:02X}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_hotkey_is_option_z() {
        let hotkey = Hotkey::from_status(0);
        assert!(hotkey.option);
        assert!(!hotkey.control);
        assert!(!hotkey.shift);
        assert_eq!(hotkey.key, KEY_Z as u8);
        assert_eq!(hotkey.format_symbols(), "⌥Z");
        assert_eq!(hotkey.format_labels(), "⌥ Option + Z");
    }

    #[test]
    fn control_shift_modifiers_only_round_trips() {
        let hotkey = Hotkey {
            key: MODIFIERS_ONLY as u8,
            control: true,
            option: false,
            command: false,
            shift: true,
            beep: true,
        };
        let status = hotkey.to_status();
        let again = Hotkey::from_status(status);
        assert_eq!(again, hotkey);
        assert_eq!(again.format_symbols(), "⌃⇧");
        assert!(status as u32 & BEEP_BIT != 0);
    }

    #[test]
    fn beep_can_toggle_without_changing_chord() {
        let base = Hotkey::from_status(i64::from(DEFAULT_HOTKEY));
        let with_beep = base.with_beep(true);
        assert!(!base.beep);
        assert!(with_beep.beep);
        assert_eq!(with_beep.key, base.key);
        assert_eq!(with_beep.option, base.option);
    }

    #[test]
    fn modifier_only_chord_keeps_modifiers_released_first() {
        let mut rec = HotkeyRecorder::new(false);
        assert_eq!(rec.flags_changed(FLAG_CONTROL), Capture::Pending);
        assert_eq!(
            rec.flags_changed(FLAG_CONTROL | FLAG_SHIFT),
            Capture::Pending
        );
        // Control released first, Shift still down.
        assert_eq!(rec.flags_changed(FLAG_SHIFT), Capture::Pending);
        let Capture::Done(hotkey) = rec.flags_changed(0) else {
            panic!("chord not finished");
        };
        assert_eq!(hotkey.format_symbols(), "⌃⇧");
        assert_eq!(hotkey.key, MODIFIERS_ONLY as u8);
    }

    #[test]
    fn key_chord_needs_a_modifier_and_esc_cancels() {
        let mut rec = HotkeyRecorder::new(true);
        assert_eq!(rec.key_down(KEY_Z as u16, 0), Capture::Pending);
        assert_eq!(rec.key_down(KEY_CODE_ESCAPE, 0), Capture::Cancel);
        let Capture::Done(hotkey) = rec.key_down(KEY_Z as u16, FLAG_OPTION | (1 << 16)) else {
            panic!("chord not finished");
        };
        assert_eq!(hotkey.format_symbols(), "⌥Z");
        assert!(hotkey.beep);
    }

    #[test]
    fn typing_keys_do_not_become_the_hotkey() {
        let mut rec = HotkeyRecorder::new(false);
        rec.flags_changed(FLAG_SHIFT);
        // Shift+A is just a capital letter: keep waiting, and forget the Shift.
        assert_eq!(rec.key_down(KEY_A as u16, FLAG_SHIFT), Capture::Pending);
        assert_eq!(rec.flags_changed(0), Capture::Pending);
        let Capture::Done(hotkey) = rec.key_down(KEY_A as u16, FLAG_CONTROL | FLAG_SHIFT) else {
            panic!("chord not finished");
        };
        assert_eq!(hotkey.format_symbols(), "⌃⇧A");
    }
}
