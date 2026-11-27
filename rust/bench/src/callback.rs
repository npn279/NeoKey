use engine_diff::{benchmark, Key};
use neokey_engine::{Event, EventState, Settings};
use neokey_mac::typing::{Input, LauncherTarget, OutputAction, Typing};
use objc2::rc::autoreleasepool;
use objc2_core_foundation::CFRetained;
use objc2_core_graphics::{
    CGEvent, CGEventField, CGEventFlags, CGEventSource, CGEventSourceStateID,
};

#[allow(dead_code)]
#[path = "../../neokey-mac/src/macos/accessibility.rs"]
mod accessibility;

// The live tap entry points are unreachable. Compile the original event builder verbatim.
fn with_state<T>(_: impl FnOnce(&mut UnusedState) -> T) -> Option<T> {
    None
}
struct UnusedState;
impl UnusedState {
    fn ensure_tap(&mut self) {
        unreachable!();
    }
    fn handle_event(
        &mut self,
        _: objc2_core_graphics::CGEventTapProxy,
        _: objc2_core_graphics::CGEventType,
        _: &CGEvent,
    ) -> bool {
        unreachable!();
    }
}

#[allow(dead_code)]
mod output {
    include!("../../neokey-mac/src/macos/event_tap.rs");

    pub(super) fn new_output() -> Output {
        let ptr = std::ptr::NonNull::new(unsafe { super::bench_new_source() })
            .expect("benchmark Core Graphics event source");
        // bench_new_source follows the Create ownership rule.
        let source = unsafe { CFRetained::from_raw(ptr) };
        Output::with_source(source)
    }
}

extern "C" {
    fn bench_use_private_source(enabled: bool);
    fn bench_new_source() -> *mut CGEventSource;
    fn bench_callback_setup(step: bool);
    fn bench_callback(event: &CGEvent) -> bool;
    fn bench_record(enabled: bool);
    fn bench_pass(event: &CGEvent);
    fn bench_posted() -> u64;
    fn bench_screen(length: &mut usize) -> *const u16;
}

pub fn configure(use_private_source: bool) {
    unsafe { bench_use_private_source(use_private_source) };
}

pub fn input_events(keys: &[Key]) -> Vec<CFRetained<CGEvent>> {
    keys.iter()
        .filter(|key| {
            matches!(key.event, Event::Keyboard) && matches!(key.state, EventState::KeyDown)
        })
        .map(|key| {
            let input =
                CGEvent::new_keyboard_event(None, key.data, true).expect("synthetic key down");
            let mut flags = match key.caps {
                1 => CGEventFlags::MaskShift,
                2 => CGEventFlags::MaskAlphaShift,
                _ => CGEventFlags::empty(),
            };
            if key.ctrl {
                flags |= CGEventFlags::MaskControl;
            }
            CGEvent::set_flags(Some(&input), flags);
            CGEvent::set_integer_value_field(
                Some(&input),
                CGEventField::EventTargetUnixProcessID,
                4242,
            );
            CGEvent::set_integer_value_field(
                Some(&input),
                CGEventField::EventSourceStateID,
                i64::from(CGEventSourceStateID::CombinedSessionState.0),
            );
            assert_eq!(
                CGEvent::integer_value_field(Some(&input), CGEventField::EventSourceStateID),
                i64::from(CGEventSourceStateID::CombinedSessionState.0),
            );
            assert_eq!(
                CGEvent::integer_value_field(Some(&input), CGEventField::EventTargetUnixProcessID),
                4242
            );
            input
        })
        .collect()
}

pub struct RustCallback {
    typing: Typing,
    output: output::Output,
    step: bool,
}

impl RustCallback {
    pub fn new(settings: &Settings, step: bool) -> Self {
        let mut typing = Typing::new(settings.clone());
        super::add_macros(typing.engine_mut());
        Self {
            typing,
            output: output::new_output(),
            step,
        }
    }

    pub fn key(&mut self, event: &CGEvent) -> bool {
        autoreleasepool(|_| {
            let field = |field| CGEvent::integer_value_field(Some(event), field);
            let _source = std::hint::black_box(field(CGEventField::EventSourceUserData));
            let _target = std::hint::black_box(field(CGEventField::EventTargetUnixProcessID));
            let key = field(CGEventField::KeyboardEventKeycode) as u16;
            let proxy = std::ptr::null_mut();
            match self.typing.process_key(Input::KeyDown {
                key,
                layout_key: key,
                flags: CGEvent::flags(Some(event)).0,
                repeat: field(CGEventField::KeyboardEventAutorepeat) != 0,
            }) {
                OutputAction::Pass => true,
                OutputAction::Suppress => false,
                OutputAction::ToggleLanguage => unreachable!("hotkeys disabled"),
                OutputAction::ExtraBackspaces(count) => {
                    self.output.extra_backspaces(proxy, count);
                    true
                }
                OutputAction::Replace(edit) => {
                    let pass = edit.replay_trigger;
                    self.output.edit(
                        proxy,
                        LauncherTarget::Regular,
                        self.step,
                        self.typing.engine().settings().vCodeTable,
                        edit,
                    );
                    pass
                }
            }
        })
    }
}

pub fn cpp_setup(settings: &Settings, step: bool) {
    benchmark::reference_setup(settings);
    unsafe { bench_callback_setup(step) }
}

pub fn cpp_key(event: &CGEvent) -> bool {
    unsafe { bench_callback(event) }
}

fn trace(
    events: &[CFRetained<CGEvent>],
    mut key: impl FnMut(&CGEvent) -> bool,
) -> (Vec<Vec<u16>>, u64) {
    unsafe { bench_record(true) };
    let mut snapshots = Vec::with_capacity(events.len());
    for event in events {
        if key(event) {
            unsafe { bench_pass(event) };
        }
        let mut length = 0;
        let ptr = unsafe { bench_screen(&mut length) };
        snapshots.push(if length == 0 {
            Vec::new()
        } else {
            unsafe { std::slice::from_raw_parts(ptr, length) }.to_vec()
        });
    }
    let count = unsafe { bench_posted() };
    unsafe { bench_record(false) };
    (snapshots, count)
}

pub fn verify_screen_model() {
    use neokey_engine::data::KEY_DELETE;

    let settings = Settings {
        vSwitchKeyStatus: 0xfe00_00fe,
        ..Settings::default()
    };
    let mut keys: Vec<_> = "as Xa".chars().map(Key::character).collect();
    keys[4].ctrl = true;
    keys.push(Key {
        data: KEY_DELETE as u16,
        ..Key::character(' ')
    });
    keys.push(Key {
        caps: 2,
        ..Key::character(' ')
    });
    let events = input_events(&keys);
    cpp_setup(&settings, false);
    let (cpp, _) = trace(&events, cpp_key);
    let mut typing = RustCallback::new(&settings, false);
    let (rust, _) = trace(&events, |event| typing.key(event));
    let expected: Vec<Vec<u16>> = ["a", "á", "á ", "á X", "á X", "á ", "á  "]
        .iter()
        .map(|screen| screen.encode_utf16().collect())
        .collect();
    assert_eq!(cpp, expected, "OpenKey screen-model smoke check");
    assert_eq!(rust, expected, "NeoKeyRust screen-model smoke check");
}

pub fn verify(
    events: &[CFRetained<CGEvent>],
    settings: &Settings,
    step: bool,
) -> (usize, u64, u64, bool) {
    cpp_setup(settings, step);
    let (cpp, cpp_count) = trace(events, cpp_key);
    let mut rust_callback = RustCallback::new(settings, step);
    let (rust, rust_count) = trace(events, |event| rust_callback.key(event));
    assert!(
        cpp_count > 0 && rust_count > 0,
        "both paths must emit CGEvents"
    );
    let mismatches = cpp.iter().zip(&rust).filter(|(a, b)| a != b).count();
    if let Some(first) = cpp.iter().zip(&rust).position(|(a, b)| a != b) {
        let event = &events[first];
        let key = CGEvent::integer_value_field(Some(event), CGEventField::KeyboardEventKeycode);
        let flags = CGEvent::flags(Some(event)).0;
        let tail =
            |screen: &[u16]| String::from_utf16_lossy(&screen[screen.len().saturating_sub(32)..]);
        eprintln!(
            "First reconstructed screen difference at input {first} (virtual key {key}, flags {flags:#x}): OpenKey {:?}, NeoKeyRust {:?}",
            tail(&cpp[first]), tail(&rust[first])
        );
    }
    (mismatches, cpp_count, rust_count, cpp.last() == rust.last())
}
