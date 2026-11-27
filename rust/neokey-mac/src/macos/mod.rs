mod accessibility;
mod event_tap;
mod input_source;
mod preferences;
mod startup_accessibility;
mod status_menu;

use std::cell::RefCell;
use std::collections::HashMap;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::rc::Rc;

use neokey_engine::vietnamese::character_to_key;
use neokey_engine::{Settings, SmartSwitch};
use neokey_mac::typing::{AppBehavior, Input, LauncherTarget, OutputAction, Typing};
use objc2::rc::{autoreleasepool, Retained};
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSBeep, NSEvent, NSRunningApplication,
    NSWorkspace, NSWorkspaceDidActivateApplicationNotification,
    NSWorkspaceDidTerminateApplicationNotification,
};
use objc2_core_graphics::{CGEvent, CGEventField, CGEventTapProxy, CGEventType};
use objc2_foundation::{ns_string, NSBundle, NSRunLoop, NSRunLoopCommonModes, NSTimer};

use self::event_tap::Tap;
use self::preferences::Preferences;
use self::startup_accessibility::{Action, StartupAccessibility};
use self::status_menu::{MenuTarget, StatusMenu};

const MAX_CACHED_PIDS: usize = 128;

thread_local! {
    static STATE: RefCell<Option<State>> = const { RefCell::new(None) };
}

fn with_state<T>(action: impl FnOnce(&mut State) -> T) -> Option<T> {
    STATE.with(|cell| {
        // AppKit and AX can enter a nested run loop. Leave nested events alone.
        cell.try_borrow_mut().ok()?.as_mut().map(action)
    })
}

pub fn run() -> Result<(), &'static str> {
    autoreleasepool(|_| {
        let mtm = MainThreadMarker::new().ok_or("NeoKeyRust requires the main thread")?;
        let trusted_at_launch = accessibility::is_trusted();
        let bundle_path = NSBundle::mainBundle()
            .bundleURL()
            .to_file_path()
            .filter(|path| {
                path.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("app"))
            });
        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
        let (preferences, settings) = Preferences::load();
        let target = MenuTarget::new(mtm);
        let menu = match StatusMenu::new(mtm, &target) {
            Ok(menu) => Some(Rc::new(menu)),
            Err(message) => {
                eprintln!("{message}");
                None
            }
        };
        let spell_preference = settings.vCheckSpelling;
        let mut typing = Typing::new(settings);
        typing.set_chromium_fix(preferences.chromium_fix);
        let macro_data = preferences.data(ns_string!("macroData"));
        if typing.engine_mut().init_macros(&macro_data).is_err() {
            let _ = typing.engine_mut().init_macros(&[]);
            eprintln!("NeoKeyRust skipped an unreadable macroData preference");
        }
        let mut smart_switch = SmartSwitch::default();
        if smart_switch
            .init(&preferences.data(ns_string!("smartSwitchKey")))
            .is_err()
        {
            smart_switch = SmartSwitch::default();
            eprintln!("NeoKeyRust skipped an unreadable smartSwitchKey preference");
        }
        STATE.with(|state| {
            state.replace(Some(State {
                typing,
                preferences,
                spell_preference,
                smart_switch,
                smart_dirty: false,
                selection_dirty: false,
                focus: None,
                bundles: HashMap::new(),
                own_pid: std::process::id() as i32,
                startup_accessibility: StartupAccessibility::new(trusted_at_launch),
                bundle_path,
                relaunch_child: None,
                tap: None,
                menu,
            }));
        });
        let workspace = NSWorkspace::sharedWorkspace();
        with_state(|state| {
            if let Some(front) = workspace.frontmostApplication() {
                state.activate_app(&front);
            }
            state.refresh_menu();
            state.refresh_input_source();
            if trusted_at_launch {
                state.install_tap();
            } else {
                accessibility::prompt_for_permission();
                eprintln!(
                    "Grant NeoKeyRust Accessibility access in System Settings to enable typing"
                );
            }
        });

        let notifications = workspace.notificationCenter();
        // Selector handlers are defined on MenuTarget and kept alive through app.run().
        let timer = unsafe {
            notifications.addObserver_selector_name_object(
                &target,
                sel!(appActivated:),
                Some(NSWorkspaceDidActivateApplicationNotification),
                None,
            );
            notifications.addObserver_selector_name_object(
                &target,
                sel!(appTerminated:),
                Some(NSWorkspaceDidTerminateApplicationNotification),
                None,
            );
            let timer = NSTimer::timerWithTimeInterval_target_selector_userInfo_repeats(
                1.0,
                &target,
                sel!(watchdog:),
                None,
                true,
            );
            NSRunLoop::mainRunLoop().addTimer_forMode(&timer, NSRunLoopCommonModes);
            timer
        };

        app.run();

        timer.invalidate();
        unsafe { notifications.removeObserver(&target) };
        with_state(State::shutdown);
        STATE.with(|state| state.replace(None));
        Ok(())
    })
}

struct Focus {
    pid: i32,
    id: Option<Rc<str>>,
    launcher: bool,
}

struct State {
    typing: Typing,
    preferences: Preferences,
    spell_preference: i64,
    smart_switch: SmartSwitch,
    smart_dirty: bool,
    selection_dirty: bool,
    focus: Option<Focus>,
    bundles: HashMap<i32, Option<Rc<str>>>,
    own_pid: i32,
    startup_accessibility: StartupAccessibility,
    bundle_path: Option<PathBuf>,
    relaunch_child: Option<Child>,
    tap: Option<Tap>,
    menu: Option<Rc<StatusMenu>>,
}

impl State {
    fn handle_event(&mut self, proxy: CGEventTapProxy, kind: CGEventType, event: &CGEvent) -> bool {
        let field = |field| CGEvent::integer_value_field(Some(event), field);
        self.focus_pid(field(CGEventField::EventTargetUnixProcessID) as i32);
        let input = match kind {
            CGEventType::KeyDown => {
                let key = field(CGEventField::KeyboardEventKeycode) as u16;
                Input::KeyDown {
                    key,
                    layout_key: if self.preferences.layout_compat {
                        layout_key(event).unwrap_or(key)
                    } else {
                        key
                    },
                    flags: CGEvent::flags(Some(event)).0,
                    repeat: field(CGEventField::KeyboardEventAutorepeat) != 0,
                }
            }
            CGEventType::KeyUp => Input::KeyUp(field(CGEventField::KeyboardEventKeycode) as u16),
            CGEventType::FlagsChanged => Input::FlagsChanged(CGEvent::flags(Some(event)).0),
            _ => Input::MouseDown,
        };
        let Some(output) = self.tap.as_ref().map(|tap| &tap.output) else {
            return true;
        };
        let action = self.typing.process_key(input);
        let event_launcher = self.focus.as_ref().is_some_and(|focus| focus.launcher);
        let launcher = action.focused_launcher(event_launcher, || {
            let (element, pid) = output.launcher_ax.focused_element()?;
            let id = Self::bundle_for_pid(&mut self.bundles, pid)?;
            Some((AppBehavior::for_bundle(&id), element))
        });
        match action {
            OutputAction::Pass => true,
            OutputAction::Suppress => false,
            OutputAction::ToggleLanguage => {
                self.toggle_language(true);
                // A modifier release must still reach the foreground app.
                kind == CGEventType::FlagsChanged
            }
            OutputAction::Replace(edit) => {
                let pass = edit.replay_trigger;
                output.edit(
                    proxy,
                    launcher,
                    self.preferences.step_by_step,
                    self.typing.engine().settings().vCodeTable,
                    edit,
                );
                // Quartz delivers tap-posted events before this returned trigger.
                pass
            }
            OutputAction::ExtraBackspaces(count) => {
                match launcher {
                    LauncherTarget::Regular => output.extra_backspaces(proxy, count),
                    // Compound Unicode is already deleted as one grapheme by launchers.
                    LauncherTarget::Focused(_) | LauncherTarget::SelectOnly
                        if self.typing.engine().settings().vCodeTable == 3 => {}
                    LauncherTarget::Focused(element)
                        if output.launcher_ax.replace(&element, count, &[]) => {}
                    LauncherTarget::Focused(_) | LauncherTarget::SelectOnly => {
                        // The physical Backspace that follows will delete the whole selection.
                        output.select_left(proxy, count + 1);
                    }
                }
                true
            }
        }
    }

    fn focus_pid(&mut self, pid: i32) {
        if pid <= 0
            || pid == self.own_pid
            || self.focus.as_ref().is_some_and(|focus| focus.pid == pid)
        {
            return;
        }
        let id = Self::bundle_for_pid(&mut self.bundles, pid);
        self.set_focus(pid, id);
    }

    fn activate_app(&mut self, app: &NSRunningApplication) {
        let pid = app.processIdentifier();
        if pid <= 0
            || pid == self.own_pid
            || self
                .focus
                .as_ref()
                .is_some_and(|focus| focus.pid == pid && focus.id.is_some())
        {
            return;
        }
        let id = Self::app_id(app);
        Self::cache_pid(&mut self.bundles, pid, id.clone());
        self.set_focus(pid, id);
    }

    fn bundle_for_pid(bundles: &mut HashMap<i32, Option<Rc<str>>>, pid: i32) -> Option<Rc<str>> {
        if let Some(id) = bundles.get(&pid) {
            return id.clone();
        }
        let id = NSRunningApplication::runningApplicationWithProcessIdentifier(pid)
            .as_deref()
            .and_then(Self::app_id);
        Self::cache_pid(bundles, pid, id.clone());
        id
    }

    fn app_id(app: &NSRunningApplication) -> Option<Rc<str>> {
        app.bundleIdentifier()
            .or_else(|| app.localizedName())
            .map(|id| id.to_string())
            .filter(|id| !id.is_empty() && id.len() <= u8::MAX as usize)
            .map(Rc::<str>::from)
    }

    fn cache_pid(bundles: &mut HashMap<i32, Option<Rc<str>>>, pid: i32, id: Option<Rc<str>>) {
        if bundles.len() >= MAX_CACHED_PIDS {
            bundles.clear();
        }
        bundles.insert(pid, id);
    }

    fn set_focus(&mut self, pid: i32, id: Option<Rc<str>>) {
        let behavior = id
            .as_deref()
            .map(AppBehavior::for_bundle)
            .unwrap_or_default();
        self.typing.set_app(behavior);
        if let Some(id) = &id {
            let mut settings = self.typing.engine().settings().clone();
            if settings.vUseSmartSwitchKey != 0 {
                let stored = self.smart_switch.get(id, selection(&settings));
                if stored == -1 {
                    self.smart_dirty = true;
                } else if (0..=9).contains(&stored) {
                    let language = i64::from(stored & 1);
                    let table = if settings.vRememberCode != 0 {
                        i64::from(stored >> 1)
                    } else {
                        settings.vCodeTable
                    };
                    if language != settings.vLanguage || table != settings.vCodeTable {
                        settings.vLanguage = language;
                        settings.vCodeTable = table;
                        self.apply_settings(settings);
                        self.selection_dirty = true;
                        self.refresh_menu();
                    }
                }
            }
        }
        self.focus = Some(Focus {
            pid,
            id,
            launcher: behavior.launcher,
        });
    }

    fn forget_app(&mut self, pid: i32) {
        self.bundles.remove(&pid);
        if self.focus.as_ref().is_some_and(|focus| focus.pid == pid) {
            self.focus = None;
            self.typing.set_app(AppBehavior::default());
        }
    }

    fn toggle_language(&mut self, shortcut: bool) {
        let mut settings = self.typing.engine().settings().clone();
        settings.vLanguage ^= 1;
        if shortcut && settings.vSwitchKeyStatus & 0x8000 != 0 {
            NSBeep();
        }
        let remember = settings.vUseSmartSwitchKey != 0;
        self.apply_settings(settings);
        if remember {
            self.remember_selection();
        }
        self.selection_dirty = true;
        self.refresh_menu();
    }

    fn select_input_type(&mut self, mode: i64) {
        let mut settings = self.typing.engine().settings().clone();
        settings.vInputType = mode;
        self.apply_settings(settings);
        self.preferences.integer(ns_string!("InputType"), mode);
        self.refresh_menu();
    }

    fn select_code_table(&mut self, table: i64) {
        let mut settings = self.typing.engine().settings().clone();
        settings.vCodeTable = table;
        let remember = settings.vUseSmartSwitchKey != 0 && settings.vRememberCode != 0;
        self.apply_settings(settings);
        if remember {
            self.remember_selection();
        }
        self.selection_dirty = true;
        self.refresh_menu();
    }

    fn toggle_spelling(&mut self) {
        self.spell_preference = i64::from(self.spell_preference == 0);
        let mut settings = self.typing.engine().settings().clone();
        settings.vCheckSpelling = self.spell_preference;
        self.apply_settings(settings);
        self.preferences
            .integer(ns_string!("Spelling"), self.spell_preference);
        self.refresh_menu();
    }

    fn apply_settings(&mut self, mut settings: Settings) {
        let was_temporary = settings.vCheckSpelling != self.spell_preference;
        settings.vCheckSpelling = self.spell_preference;
        self.typing.engine_mut().set_settings(settings);
        if was_temporary {
            self.typing.engine_mut().temp_off_spell_checking();
        }
        self.typing.reset_session();
    }

    fn refresh_menu(&self) {
        if let Some(menu) = &self.menu {
            menu.refresh(self.typing.engine().settings(), self.spell_preference != 0);
        }
    }

    fn remember_selection(&mut self) {
        if let Some(id) = self.focus.as_ref().and_then(|focus| focus.id.as_deref()) {
            self.smart_switch
                .set(id, selection(self.typing.engine().settings()));
            self.smart_dirty = true;
        }
    }

    fn flush_preferences(&mut self) {
        if std::mem::take(&mut self.selection_dirty) {
            let settings = self.typing.engine().settings();
            self.preferences
                .integer(ns_string!("InputMethod"), settings.vLanguage);
            self.preferences
                .integer(ns_string!("CodeTable"), settings.vCodeTable);
        }
        if std::mem::take(&mut self.smart_dirty) {
            self.preferences
                .set_data(ns_string!("smartSwitchKey"), &self.smart_switch.save_data());
        }
    }

    fn shutdown(&mut self) {
        self.flush_preferences();
        self.tap = None;
        if let Some(menu) = self.menu.take() {
            menu.remove();
        }
    }

    fn install_tap(&mut self) {
        self.tap = Tap::install();
        if self.tap.is_some() {
            eprintln!("NeoKeyRust keyboard event tap installed");
        }
    }

    fn ensure_tap(&mut self) {
        if self.tap.as_ref().is_some_and(Tap::ensure_enabled) {
            self.typing.reset_after_tap_disable();
        }
    }

    fn refresh_input_source(&mut self) {
        if self.typing.engine().settings().vOtherLanguage != 0 {
            self.typing
                .set_blocked_layout(input_source::is_non_english());
        }
    }

    fn watchdog(&mut self) -> bool {
        if let Some(child) = &mut self.relaunch_child {
            // Check Launch Services' result without blocking the AppKit run loop.
            match child.try_wait() {
                Ok(None) => return false,
                Ok(Some(status)) if status.success() => return true,
                Ok(Some(status)) => {
                    eprintln!("Could not relaunch NeoKeyRust ({status}). Quit and reopen NeoKeyRust.");
                }
                Err(error) => {
                    eprintln!("Could not check NeoKeyRust relaunch ({error}). Quit and reopen NeoKeyRust.");
                }
            }
            self.relaunch_child = None;
        }

        if self.tap.as_ref().is_some_and(|tap| !tap.is_valid()) {
            self.tap = None;
            self.typing.reset_after_tap_disable();
        }
        if self.tap.is_some() {
            self.ensure_tap();
        } else {
            match self
                .startup_accessibility
                .action(accessibility::is_trusted(), self.bundle_path.is_some())
            {
                Action::Wait => {}
                Action::InstallTap => self.install_tap(),
                Action::InstallTapAndRecommendRestart => {
                    eprintln!("Restart NeoKeyRust for Spotlight, Alfred and Raycast support");
                    self.install_tap();
                }
                Action::Relaunch => {
                    self.flush_preferences();
                    self.preferences.synchronize();
                    if let Some(path) = &self.bundle_path {
                        match relaunch_bundle(path) {
                            Ok(child) => self.relaunch_child = Some(child),
                            Err(error) => {
                                eprintln!("Could not relaunch NeoKeyRust ({error}). Quit and reopen NeoKeyRust.");
                            }
                        }
                    }
                }
            }
        }
        self.refresh_input_source();
        self.flush_preferences();
        false
    }
}

fn relaunch_bundle(path: &Path) -> std::io::Result<Child> {
    Command::new("/usr/bin/open")
        .args(["-g", "-n"])
        .arg(path)
        .process_group(0)
        .stdin(Stdio::null())
        .spawn()
}

fn selection(settings: &Settings) -> i32 {
    (settings.vLanguage | (settings.vCodeTable << 1)) as i32
}

fn layout_key(event: &CGEvent) -> Option<u16> {
    let text: Retained<_> = NSEvent::eventWithCGEvent(event)?.charactersIgnoringModifiers()?;
    let text = text.to_string();
    let bytes = text.as_bytes();
    if let [byte] = bytes {
        character_to_key(char::from(*byte)).map(|key| key as u16)
    } else {
        None
    }
}
