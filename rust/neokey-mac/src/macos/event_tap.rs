use std::ffi::c_void;
use std::ptr::{self, NonNull};

use neokey_engine::data::{KEY_DELETE, KEY_LEFT};
use neokey_mac::typing::{Character, Edit, LauncherTarget, Workaround};
use objc2::rc::autoreleasepool;
use objc2_application_services::AXUIElement;
use objc2_core_foundation::{
    kCFRunLoopCommonModes, CFMachPort, CFRetained, CFRunLoop, CFRunLoopSource,
};
use objc2_core_graphics::{
    CGEvent, CGEventField, CGEventFlags, CGEventSource, CGEventSourceStateID, CGEventTapLocation,
    CGEventTapOptions, CGEventTapPlacement, CGEventTapProxy, CGEventType,
};

use super::{accessibility::LauncherAccessibility, with_state};

// A private event source plus a tag avoids consuming events from other input tools.
const SYNTHETIC_TAG: i64 = 0x4e65_6f4b_6579_5253;
const BATCH_SIZE: usize = 16;

pub struct Tap {
    port: CFRetained<CFMachPort>,
    source: CFRetained<CFRunLoopSource>,
    pub output: Output,
}

impl Tap {
    pub fn install() -> Option<Self> {
        let output = Output::new()?;
        let mask = [
            CGEventType::KeyDown,
            CGEventType::KeyUp,
            CGEventType::FlagsChanged,
            CGEventType::LeftMouseDown,
            CGEventType::RightMouseDown,
            CGEventType::OtherMouseDown,
            CGEventType::LeftMouseDragged,
            CGEventType::RightMouseDragged,
        ]
        .into_iter()
        .fold(0, |mask, event| mask | (1 << event.0));
        // No refcon is needed. All callbacks are delivered on the thread's run loop.
        let port = unsafe {
            CGEvent::tap_create(
                CGEventTapLocation::SessionEventTap,
                CGEventTapPlacement::HeadInsertEventTap,
                CGEventTapOptions::Default,
                mask,
                Some(callback),
                ptr::null_mut(),
            )
        }?;
        let source = CFMachPort::new_run_loop_source(None, Some(&port), 0)?;
        CFRunLoop::current()?.add_source(Some(&source), unsafe { kCFRunLoopCommonModes });
        CGEvent::tap_enable(&port, true);
        Some(Self {
            port,
            source,
            output,
        })
    }

    pub fn is_valid(&self) -> bool {
        self.port.is_valid()
    }

    pub fn ensure_enabled(&self) -> bool {
        if CGEvent::tap_is_enabled(&self.port) {
            return false;
        }
        CGEvent::tap_enable(&self.port, true);
        true
    }
}

impl Drop for Tap {
    fn drop(&mut self) {
        self.source.invalidate();
        self.port.invalidate();
    }
}

unsafe extern "C-unwind" fn callback(
    proxy: CGEventTapProxy,
    kind: CGEventType,
    event: NonNull<CGEvent>,
    _: *mut c_void,
) -> *mut CGEvent {
    if matches!(
        kind,
        CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput
    ) {
        with_state(|state| state.ensure_tap());
        return ptr::null_mut();
    }
    // Quartz keeps the original CGEvent alive until the callback returns.
    let incoming = unsafe { event.as_ref() };
    if CGEvent::integer_value_field(Some(incoming), CGEventField::EventSourceUserData)
        == SYNTHETIC_TAG
    {
        return event.as_ptr();
    }
    let pass = autoreleasepool(|_| {
        with_state(|state| state.handle_event(proxy, kind, incoming)).unwrap_or(true)
    });
    if pass {
        event.as_ptr()
    } else {
        ptr::null_mut()
    }
}

struct KeyPair {
    down: CFRetained<CGEvent>,
    up: CFRetained<CGEvent>,
}

impl KeyPair {
    fn new(source: &CGEventSource, key: u16, shift: bool, text: &[u16]) -> Option<Self> {
        let down = CGEvent::new_keyboard_event(Some(source), key, true)?;
        let up = CGEvent::new_keyboard_event(Some(source), key, false)?;
        for event in [&*down, &*up] {
            let mut flags = CGEventFlags::MaskNonCoalesced;
            if shift {
                flags |= CGEventFlags::MaskShift;
            }
            CGEvent::set_flags(Some(event), flags);
            CGEvent::set_integer_value_field(
                Some(event),
                CGEventField::EventSourceUserData,
                SYNTHETIC_TAG,
            );
            if !text.is_empty() {
                // Quartz copies the supplied UTF-16 units into the event.
                unsafe {
                    CGEvent::keyboard_set_unicode_string(
                        Some(event),
                        text.len() as u64,
                        text.as_ptr(),
                    )
                };
            }
        }
        Some(Self { down, up })
    }

    fn post(&self, proxy: CGEventTapProxy) {
        // The proxy is live for the duration of the tap callback.
        unsafe {
            CGEvent::tap_post_event(proxy, Some(&self.down));
            CGEvent::tap_post_event(proxy, Some(&self.up));
        }
    }
}

pub struct Output {
    source: CFRetained<CGEventSource>,
    pub(super) launcher_ax: LauncherAccessibility,
    backspace: Option<KeyPair>,
}

impl Output {
    fn new() -> Option<Self> {
        let source = CGEventSource::new(CGEventSourceStateID::Private)?;
        Some(Self::with_source(source))
    }

    fn with_source(source: CFRetained<CGEventSource>) -> Self {
        CGEventSource::set_local_events_suppression_interval(Some(&source), 0.0);
        let backspace = KeyPair::new(&source, KEY_DELETE as u16, false, &[]);
        Self {
            source,
            launcher_ax: LauncherAccessibility::new(),
            backspace,
        }
    }

    pub fn extra_backspaces(&self, proxy: CGEventTapProxy, count: usize) {
        if let Some(backspace) = &self.backspace {
            for _ in 0..count {
                backspace.post(proxy);
            }
        } else {
            for _ in 0..count {
                self.send(proxy, KEY_DELETE as u16, false, &[]);
            }
        }
    }

    pub fn edit(
        &self,
        proxy: CGEventTapProxy,
        launcher: LauncherTarget<CFRetained<AXUIElement>>,
        step_by_step: bool,
        code_table: i64,
        edit: Edit,
    ) {
        if let LauncherTarget::Focused(element) = &launcher {
            let mut replacement = Vec::with_capacity(edit.characters.len());
            let all_text = edit.characters.iter().all(|character| match character {
                Character::Text { units, len } => {
                    replacement.extend_from_slice(&units[..*len]);
                    true
                }
                Character::Key { .. } => false,
            });
            if all_text
                && self
                    .launcher_ax
                    .replace(element, edit.utf16_delete, &replacement)
            {
                return;
            }
        }
        if matches!(launcher, LauncherTarget::Regular) {
            match edit.workaround {
                Workaround::None => self.extra_backspaces(proxy, edit.backspaces),
                Workaround::Placeholder(character) => {
                    self.send(proxy, 0, false, &[character]);
                    self.extra_backspaces(proxy, edit.backspaces + 1);
                }
                Workaround::SelectBeforeDelete {
                    selection,
                    backspaces,
                } => {
                    self.select_left(proxy, selection);
                    self.extra_backspaces(proxy, backspaces);
                }
            }
        } else {
            self.select_left(proxy, edit.launcher_selection);
        }

        let mut batch = [0; BATCH_SIZE];
        let mut filled = 0;
        for character in edit.characters {
            match character {
                Character::Text { units, len } if step_by_step => {
                    if len == 2 && matches!(code_table, 1 | 2 | 4) && units[0] <= 0xff {
                        // Legacy tables emit their two bytes as separate keystrokes in this mode.
                        self.send(proxy, 0, false, &units[..1]);
                        self.send(proxy, 0, false, &units[1..]);
                    } else {
                        self.send(proxy, 0, false, &units[..len]);
                    }
                }
                Character::Text { units, len } => {
                    if filled + len > batch.len() {
                        self.send(proxy, 0, false, &batch[..filled]);
                        filled = 0;
                    }
                    batch[filled..filled + len].copy_from_slice(&units[..len]);
                    filled += len;
                }
                Character::Key { code, shift } => {
                    if filled != 0 {
                        self.send(proxy, 0, false, &batch[..filled]);
                        filled = 0;
                    }
                    self.send(proxy, code, shift, &[]);
                }
            }
        }
        if filled != 0 {
            self.send(proxy, 0, false, &batch[..filled]);
        }
    }

    pub fn select_left(&self, proxy: CGEventTapProxy, count: usize) {
        for _ in 0..count {
            self.send(proxy, KEY_LEFT as u16, true, &[]);
        }
    }

    fn send(&self, proxy: CGEventTapProxy, code: u16, shift: bool, text: &[u16]) {
        if let Some(pair) = KeyPair::new(&self.source, code, shift, text) {
            pair.post(proxy);
        }
    }
}
