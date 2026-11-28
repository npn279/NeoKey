use super::{clipboard, read_app, with_app, WM_CONVERT, WM_FOREGROUND, WM_LANGUAGE_CHANGED};
use neokey_win::{
    settings::{Hotkey, Setting},
    typing::{Action, Character, Edit, Workaround},
};
use std::{mem::size_of, ptr::null_mut};
use windows_sys::Win32::{
    Foundation::*,
    UI::{
        Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
        Input::{Ime::ImmGetDefaultIMEWnd, KeyboardAndMouse::*},
        WindowsAndMessaging::*,
    },
};

const INPUT_TAG: usize = 0x4e454f4b;

pub struct Hooks {
    keyboard: HHOOK,
    mouse: HHOOK,
    foreground: HWINEVENTHOOK,
}

impl Hooks {
    pub unsafe fn install(module: HINSTANCE) -> std::io::Result<Self> {
        let hooks = Self {
            keyboard: SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard), module, 0),
            mouse: SetWindowsHookExW(WH_MOUSE_LL, Some(mouse), module, 0),
            foreground: SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                null_mut(),
                Some(foreground),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            ),
        };
        if hooks.keyboard.is_null() || hooks.mouse.is_null() || hooks.foreground.is_null() {
            Err(std::io::Error::last_os_error())
        } else {
            // Low-level callbacks see key state before the current event is applied.
            with_app(|app| {
                for vk in [
                    VK_LCONTROL,
                    VK_RCONTROL,
                    VK_LMENU,
                    VK_RMENU,
                    VK_LWIN,
                    VK_RWIN,
                    VK_LSHIFT,
                    VK_RSHIFT,
                ] {
                    if GetAsyncKeyState(i32::from(vk)) < 0 {
                        app.typing.key(vk, true, false, false);
                    }
                }
            });
            Ok(hooks)
        }
    }
}

impl Drop for Hooks {
    fn drop(&mut self) {
        unsafe {
            if !self.keyboard.is_null() {
                UnhookWindowsHookEx(self.keyboard);
            }
            if !self.mouse.is_null() {
                UnhookWindowsHookEx(self.mouse);
            }
            if !self.foreground.is_null() {
                UnhookWinEvent(self.foreground);
            }
        }
    }
}

unsafe extern "system" fn foreground(
    _: HWINEVENTHOOK,
    _: u32,
    _: HWND,
    _: i32,
    _: i32,
    _: u32,
    _: u32,
) {
    if let Some(owner) = read_app(|app| app.owner) {
        PostMessageW(owner, WM_FOREGROUND, 0, 0);
    }
}

unsafe extern "system" fn mouse(code: i32, message: WPARAM, data: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32
        && matches!(
            message as u32,
            WM_LBUTTONDOWN
                | WM_LBUTTONUP
                | WM_RBUTTONDOWN
                | WM_RBUTTONUP
                | WM_MBUTTONDOWN
                | WM_MBUTTONUP
                | WM_XBUTTONDOWN
                | WM_XBUTTONUP
                | WM_NCXBUTTONDOWN
                | WM_NCXBUTTONUP
        )
    {
        with_app(|app| app.typing.mouse_button());
    }
    CallNextHookEx(null_mut(), code, message, data)
}

unsafe fn ime_open(window: HWND) -> bool {
    // Limit the original synchronous IME query to a small fraction of the hook timeout.
    let ime = ImmGetDefaultIMEWnd(window);
    if ime.is_null() {
        return false;
    }
    let mut open = 0;
    SendMessageTimeoutW(
        ime,
        WM_IME_CONTROL,
        5, /* IMC_GETOPENSTATUS */
        0,
        SMTO_ABORTIFHUNG | SMTO_BLOCK,
        10,
        &mut open,
    );
    open != 0
}

unsafe extern "system" fn keyboard(code: i32, message: WPARAM, data: LPARAM) -> LRESULT {
    if code != HC_ACTION as i32 {
        return CallNextHookEx(null_mut(), code, message, data);
    }
    let key = &*(data as *const KBDLLHOOKSTRUCT);
    // OpenKey.cpp ignores events it injected (nonzero dwExtraInfo). Other
    // programs' synthetic keys, including on-screen keyboards, are still composed.
    if key.dwExtraInfo == INPUT_TAG {
        return CallNextHookEx(null_mut(), code, message, data);
    }
    let vk = key.vkCode as u16;
    let front = GetForegroundWindow();
    let own_ui = super::ui::is_capture_focused(front);
    // An open IME owns the key. Do not reset the composition or update modifiers.
    if !own_ui && ime_open(front) {
        return CallNextHookEx(null_mut(), code, message, data);
    }
    let down = matches!(message as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
    let caps = GetKeyState(i32::from(VK_CAPITAL)) & 1 != 0;
    let Some((owner, action, metro, beep)) = with_app(|app| {
        let action = app.typing.key(vk, down, caps, own_ui);
        let beep = Hotkey(app.typing.preferences.get(Setting::SwitchKey)).beep();
        (app.owner, action, app.typing.metro(), beep)
    }) else {
        return CallNextHookEx(null_mut(), code, message, data);
    };

    match action {
        Action::Replace(edit) => {
            apply(owner, edit);
            1
        }
        Action::Suppress => 1,
        Action::ToggleLanguage => {
            // Update the engine before the next key. Save settings and update UI in the message loop.
            with_app(|app| app.toggle_language());
            PostMessageW(owner, WM_LANGUAGE_CHANGED, usize::from(beep), 0);
            if is_modifier(vk) {
                CallNextHookEx(null_mut(), code, message, data)
            } else {
                1
            }
        }
        Action::QuickConvert => {
            PostMessageW(owner, WM_CONVERT, 0, 0);
            if is_modifier(vk) {
                CallNextHookEx(null_mut(), code, message, data)
            } else {
                1
            }
        }
        Action::ExtraBackspace => {
            backspace(metro);
            CallNextHookEx(null_mut(), code, message, data)
        }
        Action::Pass => CallNextHookEx(null_mut(), code, message, data),
    }
}

fn is_modifier(vk: u16) -> bool {
    matches!(
        vk,
        VK_LSHIFT | VK_RSHIFT | VK_LCONTROL | VK_RCONTROL | VK_LMENU | VK_RMENU | VK_LWIN | VK_RWIN
    )
}

fn input(vk: u16, unit: u16, flags: u32) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: unit,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: INPUT_TAG,
            },
        },
    }
}

unsafe fn send(events: &[INPUT]) {
    SendInput(
        events.len() as u32,
        events.as_ptr(),
        size_of::<INPUT>() as i32,
    );
}

unsafe fn key(vk: u16) {
    if vk != 0 {
        let flags = if matches!(
            vk,
            VK_LEFT
                | VK_UP
                | VK_RIGHT
                | VK_DOWN
                | VK_INSERT
                | VK_DELETE
                | VK_HOME
                | VK_END
                | VK_PRIOR
                | VK_NEXT
        ) {
            KEYEVENTF_EXTENDEDKEY
        } else {
            0
        };
        send(&[input(vk, 0, flags), input(vk, 0, flags | KEYEVENTF_KEYUP)]);
    }
}

unsafe fn unicode(units: &[u16]) {
    // A group has up to two encoded units, or one supplementary character.
    let mut events = [input(0, 0, 0); 4];
    for (unit, pair) in units.iter().zip(events.as_chunks_mut::<2>().0) {
        pair[0] = input(0, *unit, KEYEVENTF_UNICODE);
        pair[1] = input(0, *unit, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP);
    }
    send(&events[..units.len() * 2]);
}

unsafe fn shift_chord(vk: u16) {
    // Physical Shift may already be held during composition.
    if GetAsyncKeyState(i32::from(VK_SHIFT)) < 0 {
        key(vk);
        return;
    }
    send(&[
        input(VK_LSHIFT, 0, 0),
        input(vk, 0, KEYEVENTF_EXTENDEDKEY),
        input(vk, 0, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP),
        input(VK_LSHIFT, 0, KEYEVENTF_KEYUP),
    ]);
}

pub unsafe fn metro_backspace() {
    // These backspaces must land before the replacement character. A posted
    // broadcast can be handled later and delete that character. The wait is
    // bounded so a hung window cannot remove the keyboard hook.
    let flags = SMTO_ABORTIFHUNG | SMTO_BLOCK;
    let mut handled = 0;
    SendMessageTimeoutW(
        HWND_BROADCAST,
        WM_CHAR,
        usize::from(VK_BACK),
        0,
        flags,
        50,
        &mut handled,
    );
    SendMessageTimeoutW(
        HWND_BROADCAST,
        WM_CHAR,
        usize::from(VK_BACK),
        0,
        flags,
        50,
        &mut handled,
    );
}

unsafe fn backspace(metro: bool) {
    key(VK_BACK);
    if metro {
        metro_backspace();
    }
}

unsafe fn flush(owner: HWND, text: &mut Vec<u16>) {
    if text.is_empty() {
        return;
    }
    if clipboard::paste_text(owner, text) {
        shift_chord(VK_INSERT);
    } else {
        let mut remaining = text.as_slice();
        while let Some((&unit, tail)) = remaining.split_first() {
            let len = usize::from(
                (0xd800..=0xdbff).contains(&unit)
                    && tail
                        .first()
                        .is_some_and(|next| (0xdc00..=0xdfff).contains(next)),
            ) + 1;
            unicode(&remaining[..len]);
            remaining = &remaining[len..];
        }
    }
    text.clear();
}

unsafe fn apply(owner: HWND, edit: Edit) {
    match edit.workaround {
        Workaround::None => {}
        Workaround::Placeholder => unicode(&[0x202f]),
        Workaround::SelectLeft => shift_chord(VK_LEFT),
    }
    for _ in 0..edit.backspaces {
        backspace(edit.metro);
    }
    if edit.step_by_step {
        let mut groups = edit.characters.iter().peekable();
        while let Some(group) = groups.next() {
            match *group {
                Character::Key(vk) => key(vk),
                Character::Text([first, _], 1) if (0xd800..=0xdbff).contains(&first) => {
                    match groups.peek() {
                        Some(Character::Text([second, _], 1))
                            if (0xdc00..=0xdfff).contains(second) =>
                        {
                            unicode(&[first, *second]);
                            groups.next();
                        }
                        _ => unicode(&[first]),
                    }
                }
                Character::Text(units, len) => unicode(&units[..len.into()]),
            }
        }
    } else {
        let mut text = Vec::with_capacity(edit.characters.len());
        for group in edit.characters {
            match group {
                Character::Key(vk) => {
                    flush(owner, &mut text);
                    key(vk);
                }
                Character::Text(units, len) => text.extend_from_slice(&units[..len.into()]),
            }
        }
        flush(owner, &mut text);
    }
    if let Some(trigger) = edit.macro_trigger {
        match trigger {
            Character::Key(vk) => key(vk),
            Character::Text(units, len) => unicode(&units[..len.into()]),
        }
    }
}
