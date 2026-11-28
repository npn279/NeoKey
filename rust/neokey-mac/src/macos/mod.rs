mod accessibility;
mod app_actions;
mod clipboard;
mod event_tap;
mod input_source;
mod instance;
mod login_item;
mod preferences;
mod startup_accessibility;
mod status_menu;
mod ui;
mod updates;

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
use self::login_item::LoginStatus;
use self::preferences::Preferences;
use self::startup_accessibility::{Action, StartupAccessibility};
use self::status_menu::{MenuTarget, StatusMenu};
use self::ui::{
    alert, tag, Capture, Hotkey, HotkeyRecorder, SettingsSection, Ui, UiHandle, UiSnapshot,
};
use self::updates::Update;

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
        let mtm = MainThreadMarker::new().ok_or("NeoKey requires the main thread")?;
        let trusted_at_launch = accessibility::is_trusted();
        let bundle_path = NSBundle::mainBundle()
            .bundleURL()
            .to_file_path()
            .filter(|path| {
                path.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("app"))
            });
        let snapshot_dir = std::env::var("NEOKEY_UI_SNAPSHOT").ok();
        let instance_lock = if bundle_path.is_some() && snapshot_dir.is_none() {
            match instance::acquire() {
                Ok(Some(lock)) => Some(lock),
                Ok(None) => return Ok(()),
                Err(error) => {
                    eprintln!("Could not acquire NeoKey's instance lock: {error}");
                    return Err("NeoKey could not start");
                }
            }
        } else {
            None
        };
        let app = NSApplication::sharedApplication(mtm);
        let first_login_setting = snapshot_dir
            .is_none()
            .then(preferences::migrate_legacy_defaults)
            .flatten();
        let (preferences, settings) = Preferences::load();
        let dock_icon = preferences.bool(ns_string!("vShowIconOnDock"));
        app.setActivationPolicy(if dock_icon {
            NSApplicationActivationPolicy::Regular
        } else {
            NSApplicationActivationPolicy::Accessory
        });
        let target = MenuTarget::new(mtm);
        MenuTarget::set_app_delegate(&target, app.as_ref());
        let target_keep = target.clone();
        // Debug-only UI snapshot mode (`NEOKEY_UI_SNAPSHOT=<dir>`): render the
        // UI to PNGs and exit. No status item, event tap or onboarding.
        let menu = if snapshot_dir.is_some() {
            None
        } else {
            match StatusMenu::new(mtm, &target) {
                Ok(menu) => Some(Rc::new(menu)),
                Err(message) => {
                    eprintln!("{message}");
                    None
                }
            }
        };
        let ui = Rc::new(Ui::new(mtm, &target));
        let spell_preference = settings.vCheckSpelling;
        let convert_hotkey = preferences.get(ns_string!("convertToolHotKey"));
        ui.settings
            .convert
            .set_options(app_actions::load_convert_options(&preferences));
        let login_status = if snapshot_dir.is_none() {
            login_item::status(bundle_path.is_some())
        } else {
            LoginStatus::Unavailable
        };
        let mut typing = Typing::new(settings);
        typing.set_chromium_fix(preferences.chromium_fix);
        typing.set_convert_hotkey(convert_hotkey);
        let macro_data = preferences.data(ns_string!("macroData"));
        if typing.engine_mut().init_macros(&macro_data).is_err() {
            let _ = typing.engine_mut().init_macros(&[]);
            eprintln!("NeoKey skipped an unreadable macroData preference");
        }
        let mut smart_switch = SmartSwitch::default();
        if smart_switch
            .init(&preferences.data(ns_string!("smartSwitchKey")))
            .is_err()
        {
            smart_switch = SmartSwitch::default();
            eprintln!("NeoKey skipped an unreadable smartSwitchKey preference");
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
                instance_lock,
                relaunch_child: None,
                tap: None,
                menu,
                ui: Some(ui),
                hotkey_capture: None,
                convert_hotkey,
                login_status,
                update_check: None,
                quick_convert_pending: false,
                ui_refresh_pending: false,
                target: target_keep,
            }));
        });
        let workspace = NSWorkspace::sharedWorkspace();
        with_state(|state| {
            if let Some(front) = workspace.frontmostApplication() {
                state.activate_app(&front);
            }
            state.refresh_ui();
            state.refresh_input_source();
            if let Some(dir) = &snapshot_dir {
                // Debug-only: render the UI to PNGs and exit before any tap or
                // onboarding. `refresh` re-resolves the accent per appearance.
                if let Some(ui) = state.ui.clone() {
                    ui::snapshot::run(dir, &ui, || state.refresh_ui());
                }
                std::process::exit(0);
            }
            if let Some(enabled) = first_login_setting {
                if let Err(error) = login_item::set_enabled(state.bundle_path.is_some(), enabled) {
                    eprintln!("NeoKey could not apply launch at login: {error}");
                }
                state.refresh_login();
            }
            if state.typing.engine().settings().vSwitchKeyStatus & 0x8000 != 0 {
                NSBeep();
            }
            if trusted_at_launch {
                state.install_tap();
                if state.tap.is_none() || state.preferences.bool(ns_string!("ShowUIOnStartup")) {
                    state.show_settings(SettingsSection::General);
                }
                state.check_updates_on_startup();
            } else {
                if let Some(ui) = &state.ui {
                    ui.show_onboarding_if_needed(false);
                }
                eprintln!("Grant NeoKey Accessibility access in System Settings to enable typing");
            }
            state.refresh_ui();
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum Shortcut {
    Language,
    Convert,
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
    instance_lock: Option<std::fs::File>,
    relaunch_child: Option<Child>,
    tap: Option<Tap>,
    menu: Option<Rc<StatusMenu>>,
    ui: Option<UiHandle>,
    hotkey_capture: Option<(Shortcut, HotkeyRecorder)>,
    convert_hotkey: i64,
    login_status: LoginStatus,
    update_check: Option<(std::sync::mpsc::Receiver<Update>, bool)>,
    quick_convert_pending: bool,
    ui_refresh_pending: bool,
    target: Retained<MenuTarget>,
}

impl State {
    fn handle_event(&mut self, proxy: CGEventTapProxy, kind: CGEventType, event: &CGEvent) -> bool {
        let field = |field| CGEvent::integer_value_field(Some(event), field);
        if self.hotkey_capture.is_some() {
            if self
                .ui
                .as_ref()
                .is_some_and(|ui| ui.settings.window().isKeyWindow())
            {
                return self.capture_event(kind, event);
            }
            self.hotkey_capture = None;
            self.typing.reset_after_tap_disable();
            self.schedule_ui_refresh();
        }
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
            OutputAction::QuickConvert => {
                self.typing.reset_session();
                self.schedule_quick_convert();
                kind == CGEventType::FlagsChanged
            }
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
                        self.schedule_ui_refresh();
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
        self.schedule_ui_refresh();
    }

    fn select_input_type(&mut self, mode: i64) {
        let mut settings = self.typing.engine().settings().clone();
        settings.vInputType = mode;
        self.apply_settings(settings);
        self.preferences.integer(ns_string!("InputType"), mode);
        self.refresh_ui();
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
        self.refresh_ui();
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

    fn ui_snapshot(&self) -> UiSnapshot {
        let settings = self.typing.engine().settings().clone();
        let macros = self
            .typing
            .engine()
            .macros()
            .all()
            .map(|(_, m)| (m.text.clone(), m.content.clone()))
            .collect();
        UiSnapshot {
            settings,
            spelling: self.spell_preference != 0,
            step_by_step: self.preferences.step_by_step,
            layout_compat: self.preferences.layout_compat,
            chromium_fix: self.preferences.chromium_fix,
            hotkey: Hotkey::from_status(self.typing.engine().settings().vSwitchKeyStatus),
            macros,
            convert_hotkey: Hotkey::from_convert_status(self.convert_hotkey),
            convert_alert: self
                .preferences
                .bool(ns_string!("convertToolDontAlertWhenCompleted")),
            login_status: self.login_status,
            gray_icon: self.gray_icon(),
            dock_icon: self.preferences.bool(ns_string!("vShowIconOnDock")),
            show_on_startup: self.preferences.bool(ns_string!("ShowUIOnStartup")),
            update_on_startup: !self.preferences.bool(ns_string!("DontCheckUpdate")),
            checking_updates: self.update_check.is_some(),
            tap_ready: self.tap.is_some(),
        }
    }

    fn refresh_ui(&self) {
        if let Some(menu) = &self.menu {
            menu.refresh_title(
                self.typing.engine().settings().vLanguage != 0,
                self.gray_icon(),
            );
            menu.refresh_quick_item(&ui::style::ns(
                &Hotkey::from_convert_status(self.convert_hotkey).quick_convert_title(),
            ));
        }
        let Some(ui) = &self.ui else {
            return;
        };
        let snap = self.ui_snapshot();
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        // Re-resolve the accent on every refresh so light/dark switches stick.
        let dark = ui::style::is_dark(&NSApplication::sharedApplication(mtm).effectiveAppearance());
        ui.refresh(&snap, dark);
        let rows: Vec<_> = self
            .smart_switch
            .iter()
            .map(|(bundle, stored)| {
                let compat = AppBehavior::for_bundle(bundle).launcher;
                (bundle.to_string(), stored, compat)
            })
            .collect();
        ui.settings
            .apps
            .refresh_apps(mtm, &self.target, &rows, dark);
    }

    fn schedule_ui_refresh(&mut self) {
        if std::mem::replace(&mut self.ui_refresh_pending, true) {
            return;
        }
        alert::defer(|| {
            with_state(|state| {
                state.ui_refresh_pending = false;
                state.refresh_ui();
            });
        });
    }

    fn gray_icon(&self) -> bool {
        self.preferences.value_or(ns_string!("GrayIcon"), true)
    }

    fn show_settings(&mut self, section: SettingsSection) {
        self.refresh_login();
        if let Some(ui) = &self.ui {
            ui.panel.close();
            ui.show_settings(section);
        }
    }

    fn show_panel(&mut self, button: &objc2_app_kit::NSStatusBarButton) {
        self.refresh_login();
        if let Some(ui) = &self.ui {
            self.refresh_ui();
            ui.show_panel(button);
        }
    }

    fn set_language(&mut self, vi: bool) {
        let mut settings = self.typing.engine().settings().clone();
        let want = i64::from(vi);
        if settings.vLanguage == want {
            return;
        }
        settings.vLanguage = want;
        self.apply_settings(settings);
        if self.typing.engine().settings().vUseSmartSwitchKey != 0 {
            self.remember_selection();
        }
        self.selection_dirty = true;
        self.refresh_ui();
    }

    fn handle_ui_action(&mut self, tag: isize, sender: &objc2_app_kit::NSControl) {
        use objc2_app_kit::{NSApplication, NSPopUpButton, NSSegmentedControl, NSSwitch};
        let switch_on = || {
            sender
                .downcast_ref::<NSSwitch>()
                .is_some_and(ui::style::switch_is_on)
        };

        match tag {
            tag::LANG_SEGMENT => {
                if let Some(segment) = sender.downcast_ref::<NSSegmentedControl>() {
                    let vi = segment.selectedSegment() == 0;
                    self.set_language(vi);
                }
            }
            tag::QUIT => {
                self.shutdown();
                let mtm = MainThreadMarker::new().expect("main thread");
                NSApplication::sharedApplication(mtm).terminate(None);
            }
            tag::GEAR => self.show_settings(SettingsSection::General),
            tag::OPEN_CONVERT => self.show_settings(SettingsSection::Convert),
            tag::OPEN_MACROS => self.show_settings(SettingsSection::Macros),
            tag::OPEN_ABOUT => self.show_settings(SettingsSection::About),
            tag::HOTKEY_CAPTURE => self.toggle_hotkey_capture(Shortcut::Language),
            tag::CONVERT_HOTKEY => self.toggle_hotkey_capture(Shortcut::Convert),
            tag::CONVERT_CLEAR_HOTKEY => self.set_convert_hotkey(0),
            tag::CONVERT_QUICK => self.schedule_quick_convert(),
            tag::CONVERT_OPTIONS => self.save_convert_options(),
            tag::CONVERT_ALERT => {
                self.preferences.integer(
                    ns_string!("convertToolDontAlertWhenCompleted"),
                    i64::from(switch_on()),
                );
            }
            tag::LOGIN => self.set_login(switch_on()),
            tag::OPEN_LOGIN_SETTINGS => login_item::open_settings(),
            tag::GRAY_ICON => {
                self.preferences
                    .integer(ns_string!("GrayIcon"), i64::from(switch_on()));
                self.refresh_ui();
            }
            tag::DOCK_ICON => {
                self.preferences
                    .integer(ns_string!("vShowIconOnDock"), i64::from(switch_on()));
                if !switch_on() {
                    self.close_settings();
                }
            }
            tag::STARTUP_UI => self
                .preferences
                .integer(ns_string!("ShowUIOnStartup"), i64::from(switch_on())),
            tag::STARTUP_UPDATE => self
                .preferences
                .integer(ns_string!("DontCheckUpdate"), i64::from(!switch_on())),
            tag::UPDATE => self.check_updates(true),
            tag::RESET => self.confirm_reset(),
            tag::CLOSE_SETTINGS => self.close_settings(),
            tag::PANEL_SPELL => {
                if let Some(sw) = sender.downcast_ref::<NSSwitch>() {
                    let on = sw.state() == objc2_app_kit::NSControlStateValueOn;
                    self.spell_preference = i64::from(on);
                    let mut settings = self.typing.engine().settings().clone();
                    settings.vCheckSpelling = self.spell_preference;
                    self.apply_settings(settings);
                    self.preferences
                        .integer(ns_string!("Spelling"), self.spell_preference);
                    self.refresh_ui();
                }
            }
            tag::SMART_SWITCH => {
                if let Some(sw) = sender.downcast_ref::<NSSwitch>() {
                    let on = sw.state() == objc2_app_kit::NSControlStateValueOn;
                    let mut settings = self.typing.engine().settings().clone();
                    let smart = i64::from(on);
                    settings.vUseSmartSwitchKey = smart;
                    self.apply_settings(settings);
                    self.preferences
                        .integer(ns_string!("UseSmartSwitchKey"), smart);
                    self.refresh_ui();
                }
            }
            tag::CODE_TABLE => {
                if let Some(pop) = sender.downcast_ref::<NSPopUpButton>() {
                    self.select_code_table(pop.indexOfSelectedItem() as i64);
                }
            }
            tag::INPUT_TYPE => {
                // The panel uses a popup, the settings window a segmented control.
                let mode = if let Some(pop) = sender.downcast_ref::<NSPopUpButton>() {
                    pop.indexOfSelectedItem() as i64
                } else if let Some(segment) = sender.downcast_ref::<NSSegmentedControl>() {
                    segment.selectedSegment() as i64
                } else {
                    return;
                };
                self.select_input_type(mode);
            }
            t if (tag::SIDEBAR..tag::SIDEBAR + 6).contains(&t) => {
                self.end_hotkey_capture();
                self.show_settings(SettingsSection::ALL[(t - tag::SIDEBAR) as usize]);
            }
            tag::HOTKEY_BEEP => {
                if let Some(sw) = sender.downcast_ref::<NSSwitch>() {
                    let on = sw.state() == objc2_app_kit::NSControlStateValueOn;
                    let mut hk =
                        Hotkey::from_status(self.typing.engine().settings().vSwitchKeyStatus);
                    hk = hk.with_beep(on);
                    self.apply_hotkey(hk);
                }
            }
            t if t == tag::SETTING + 1 => {
                if let Some(sw) = sender.downcast_ref::<NSSwitch>() {
                    self.preferences.step_by_step =
                        sw.state() == objc2_app_kit::NSControlStateValueOn;
                    self.preferences.integer(
                        ns_string!("SendKeyStepByStep"),
                        i64::from(self.preferences.step_by_step),
                    );
                    self.refresh_ui();
                }
            }
            t if t == tag::SETTING + 2 => {
                if let Some(sw) = sender.downcast_ref::<NSSwitch>() {
                    self.preferences.layout_compat =
                        sw.state() == objc2_app_kit::NSControlStateValueOn;
                    self.preferences.integer(
                        ns_string!("vPerformLayoutCompat"),
                        i64::from(self.preferences.layout_compat),
                    );
                    self.refresh_ui();
                }
            }
            t if t == tag::SETTING + 3 => {
                if let Some(sw) = sender.downcast_ref::<NSSwitch>() {
                    self.preferences.chromium_fix =
                        sw.state() == objc2_app_kit::NSControlStateValueOn;
                    self.preferences.integer(
                        ns_string!("vFixChromiumBrowser"),
                        i64::from(self.preferences.chromium_fix),
                    );
                    self.typing.set_chromium_fix(self.preferences.chromium_fix);
                    self.refresh_ui();
                }
            }
            tag::MACRO_ADD => {
                if let Some(ui) = &self.ui {
                    let key = ui.settings.macros.shortcut_text();
                    let content = ui.settings.macros.content_text();
                    if !key.is_empty() && !content.is_empty() {
                        self.typing.engine_mut().add_macro(&key, &content);
                        ui.settings.macros.clear_fields();
                        self.persist_macros();
                    }
                }
            }
            t if (tag::MACRO_ROW_DELETE..tag::MACRO_ROW_DELETE + 2000).contains(&t) => {
                let idx = (t - tag::MACRO_ROW_DELETE) as usize;
                let key = self
                    .typing
                    .engine()
                    .macros()
                    .all()
                    .nth(idx)
                    .map(|(_, m)| m.text.clone());
                if let Some(key) = key {
                    self.typing.engine_mut().delete_macro(&key);
                    self.persist_macros();
                }
            }
            tag::MACRO_IMPORT => self.import_macros(),
            tag::MACRO_EXPORT => self.export_macros(),
            tag::CONVERT_RUN => self.run_convert(),
            tag::CONVERT_PASTE => self.convert_paste(),
            tag::CONVERT_COPY => self.convert_copy(),
            tag::CONVERT_SWAP => {
                if let Some(ui) = &self.ui {
                    ui.settings
                        .convert
                        .set_options(ui.settings.convert.ui_options().swap_tables());
                    self.save_convert_options();
                }
            }
            tag::ONBOARDING_OPEN => {
                alert::open_url(
                    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
                );
                if let Some(ui) = &self.ui {
                    ui.onboarding.set_waiting();
                }
            }
            tag::ONBOARDING_RETRY => {
                if !accessibility::is_trusted() {
                    if let Some(ui) = &self.ui {
                        ui.onboarding.set_waiting();
                    }
                    accessibility::prompt_for_permission();
                }
            }
            tag::ONBOARDING_QUIT => {
                self.shutdown();
                let mtm = MainThreadMarker::new().expect("main thread");
                NSApplication::sharedApplication(mtm).terminate(None);
            }
            t if (tag::LINK..tag::LINK + 7).contains(&t) => {
                let links = [
                    "https://github.com/npn279/NeoKey",
                    "https://github.com/npn279/NeoKey/issues",
                    updates::RELEASES,
                    "https://open-key.org",
                    "https://www.facebook.com/OpenKeyVN",
                    "mailto:maivutuyen.91@gmail.com",
                    "https://github.com/tuyenvm/OpenKey",
                ];
                alert::open_url(links[(t - tag::LINK) as usize]);
            }
            t if (tag::APP_LANG..tag::APP_LANG + 2000).contains(&t) => {
                if let Some(segment) = sender.downcast_ref::<NSSegmentedControl>() {
                    let bundles: Vec<_> = self
                        .smart_switch
                        .iter()
                        .map(|(b, _)| b.to_string())
                        .collect();
                    let bundle_idx = (t - tag::APP_LANG) as usize;
                    if let Some(bundle) = bundles.get(bundle_idx) {
                        let lang = i64::from(segment.selectedSegment() == 0);
                        let stored = self
                            .smart_switch
                            .iter()
                            .find(|(b, _)| *b == bundle.as_str())
                            .map(|(_, v)| v)
                            .unwrap_or(0);
                        let table = SmartSwitch::code_table(stored);
                        self.smart_switch
                            .set(bundle, SmartSwitch::packed(lang, table));
                        self.smart_dirty = true;
                        self.flush_preferences();
                        self.refresh_ui();
                    }
                }
            }
            _ => {
                if (tag::SETTING + 10..tag::SETTING + 27).contains(&tag) {
                    self.apply_typing_toggle(tag - tag::SETTING, sender);
                }
            }
        }
    }

    /// Clicking the record button starts a capture; clicking it again cancels.
    fn toggle_hotkey_capture(&mut self, shortcut: Shortcut) {
        if self.tap.is_none()
            || self
                .hotkey_capture
                .is_some_and(|(active, _)| active == shortcut)
        {
            // Without the event tap (no Accessibility access) nothing can record.
            self.end_hotkey_capture();
            return;
        }
        self.refresh_ui();
        let beep = shortcut == Shortcut::Language
            && Hotkey::from_status(self.typing.engine().settings().vSwitchKeyStatus).beep;
        self.hotkey_capture = Some((shortcut, HotkeyRecorder::new(beep)));
        if let Some(ui) = &self.ui {
            match shortcut {
                Shortcut::Language => ui.settings.general.set_capturing(),
                Shortcut::Convert => ui.settings.convert.set_capturing(),
            }
        }
    }

    /// Records the chord from the event tap itself. A nested AppKit event loop
    /// only sees keys sent to the settings window, and the tap would still act on
    /// the current hotkey (toggling the language instead of recording it).
    fn capture_event(&mut self, kind: CGEventType, event: &CGEvent) -> bool {
        let Some((_, recorder)) = self.hotkey_capture.as_mut() else {
            return true;
        };
        let flags = CGEvent::flags(Some(event)).0;
        let result = match kind {
            CGEventType::KeyDown => {
                let key =
                    CGEvent::integer_value_field(Some(event), CGEventField::KeyboardEventKeycode);
                recorder.key_down(key as u16, flags)
            }
            CGEventType::FlagsChanged => recorder.flags_changed(flags),
            CGEventType::KeyUp => {
                let key =
                    CGEvent::integer_value_field(Some(event), CGEventField::KeyboardEventKeycode);
                return self.typing.process_key(Input::KeyUp(key as u16)) == OutputAction::Pass;
            }
            _ => return true,
        };
        match result {
            Capture::Pending => {}
            Capture::Done(hotkey) => self.finish_capture(Some(hotkey)),
            Capture::Cancel => self.finish_capture(None),
        }
        if kind == CGEventType::KeyDown {
            let key = CGEvent::integer_value_field(Some(event), CGEventField::KeyboardEventKeycode);
            self.typing.consume_key_until_release(key as u16);
        }
        // Modifier changes still pass so no app is left with a stuck modifier.
        kind == CGEventType::FlagsChanged
    }

    fn finish_capture(&mut self, chord: Option<Hotkey>) {
        let Some((shortcut, _)) = self.hotkey_capture.take() else {
            return;
        };
        self.typing.reset_after_tap_disable();
        alert::defer(move || {
            with_state(|state| match (shortcut, chord) {
                (Shortcut::Language, Some(hotkey)) => state.apply_hotkey(hotkey),
                (Shortcut::Convert, Some(hotkey)) => state.set_convert_hotkey(hotkey.to_status()),
                (_, None) => state.refresh_ui(),
            });
        });
    }

    fn end_hotkey_capture(&mut self) {
        if self.hotkey_capture.take().is_some() {
            self.typing.reset_after_tap_disable();
            self.refresh_ui();
        }
    }

    fn set_convert_hotkey(&mut self, status: i64) {
        self.end_hotkey_capture();
        self.convert_hotkey = status;
        self.typing.set_convert_hotkey(status);
        self.preferences
            .integer(ns_string!("convertToolHotKey"), status);
        self.refresh_ui();
    }

    fn apply_hotkey(&mut self, hotkey: Hotkey) {
        let status = hotkey.to_status();
        let mut settings = self.typing.engine().settings().clone();
        settings.vSwitchKeyStatus = status;
        self.apply_settings(settings);
        self.preferences
            .integer(ns_string!("SwitchKeyStatus"), status);
        self.refresh_ui();
    }

    fn apply_typing_toggle(&mut self, id: isize, sender: &objc2_app_kit::NSControl) {
        use objc2_app_kit::NSSwitch;
        let on = sender
            .downcast_ref::<NSSwitch>()
            .is_some_and(|sw| sw.state() == objc2_app_kit::NSControlStateValueOn);
        let v = i64::from(on);
        let mut settings = self.typing.engine().settings().clone();
        let key = match id {
            10 => {
                self.spell_preference = v;
                settings.vCheckSpelling = v;
                self.preferences.integer(ns_string!("Spelling"), v);
                None
            }
            11 => {
                settings.vUseModernOrthography = v;
                Some("ModernOrthography")
            }
            12 => {
                settings.vQuickTelex = v;
                Some("QuickTelex")
            }
            13 => {
                settings.vRestoreIfWrongSpelling = v;
                Some("RestoreIfInvalidWord")
            }
            14 => {
                settings.vUpperCaseFirstChar = v;
                Some("UpperCaseFirstChar")
            }
            15 => {
                settings.vAllowConsonantZFWJ = v;
                Some("vAllowConsonantZFWJ")
            }
            16 => {
                settings.vQuickStartConsonant = v;
                Some("vQuickStartConsonant")
            }
            17 => {
                settings.vQuickEndConsonant = v;
                Some("vQuickEndConsonant")
            }
            18 => {
                settings.vUseMacro = v;
                Some("UseMacro")
            }
            19 => {
                settings.vUseMacroInEnglishMode = v;
                Some("UseMacroInEnglishMode")
            }
            20 => {
                settings.vAutoCapsMacro = v;
                Some("vAutoCapsMacro")
            }
            21 => {
                settings.vOtherLanguage = v;
                Some("vOtherLanguage")
            }
            22 => {
                settings.vTempOffOpenKey = v;
                Some("vTempOffOpenKey")
            }
            23 => {
                settings.vFreeMark = v;
                Some("FreeMark")
            }
            24 => {
                settings.vTempOffSpelling = v;
                Some("vTempOffSpelling")
            }
            25 => {
                settings.vFixRecommendBrowser = v;
                Some("FixRecommendBrowser")
            }
            26 => {
                settings.vRememberCode = v;
                Some("vRememberCode")
            }
            _ => None,
        };
        self.apply_settings(settings);
        if let Some(key) = key {
            self.preferences
                .integer(&objc2_foundation::NSString::from_str(key), v);
        }
        if id == 21 {
            self.refresh_input_source();
        }
        self.refresh_ui();
    }

    fn persist_macros(&mut self) {
        let bytes = self.typing.engine().macros().save_data();
        self.preferences.set_data(ns_string!("macroData"), &bytes);
        self.refresh_ui();
    }

    fn import_macros(&mut self) {
        use objc2_app_kit::{NSModalResponseOK, NSOpenPanel};
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let panel = NSOpenPanel::openPanel(mtm);
        panel.setCanChooseFiles(true);
        panel.setAllowsMultipleSelection(false);
        if panel.runModal() == NSModalResponseOK {
            let urls = panel.URLs();
            if let Some(url) = urls.firstObject() {
                let path = url.path().map(|p| p.to_string());
                if let Some(path) = path {
                    let alert = objc2_app_kit::NSAlert::new(mtm);
                    alert.setMessageText(ns_string!("Giữ lại dữ liệu gõ tắt hiện tại?"));
                    alert.addButtonWithTitle(ns_string!("Giữ lại"));
                    alert.addButtonWithTitle(ns_string!("Thay thế"));
                    alert.addButtonWithTitle(ns_string!("Huỷ"));
                    let response = alert.runModal();
                    let append = match response {
                        value if value == objc2_app_kit::NSAlertFirstButtonReturn => true,
                        value if value == objc2_app_kit::NSAlertSecondButtonReturn => false,
                        _ => return,
                    };
                    match self
                        .typing
                        .engine_mut()
                        .read_macros_from_file(&path, append)
                    {
                        Ok(()) => self.persist_macros(),
                        Err(error) => alert::ask(
                            None,
                            format!("Không thể nhập dữ liệu gõ tắt\n{error}"),
                            "",
                            "OK",
                            None,
                        ),
                    }
                }
            }
        }
        let _ = mtm;
    }

    fn export_macros(&mut self) {
        use objc2_app_kit::{NSModalResponseOK, NSSavePanel};
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let panel = NSSavePanel::savePanel(mtm);
        panel.setNameFieldStringValue(ns_string!("NeoKeyMacro.txt"));
        if panel.runModal() == NSModalResponseOK {
            if let Some(url) = panel.URL() {
                let path = url.path().map(|p| p.to_string());
                if let Some(path) = path {
                    if let Err(error) = self.typing.engine().macros().save_to_file(&path) {
                        alert::ask(
                            None,
                            format!("Không thể xuất dữ liệu gõ tắt\n{error}"),
                            "",
                            "OK",
                            None,
                        );
                    }
                }
            }
        }
    }

    fn run_convert(&mut self) {
        let Some(ui) = &self.ui else {
            return;
        };
        let text = ui.settings.convert.input_text();
        let out = ui.settings.convert.ui_options().convert(&text);
        ui.settings.convert.set_output(&out);
    }

    fn convert_paste(&mut self) {
        use objc2_app_kit::NSPasteboard;
        let pb = NSPasteboard::generalPasteboard();
        if let Some(s) = pb.stringForType(unsafe { objc2_app_kit::NSPasteboardTypeString }) {
            if let Some(ui) = &self.ui {
                ui.settings.convert.set_input(&s.to_string());
                self.run_convert();
            }
        }
    }

    fn convert_copy(&mut self) {
        use objc2_app_kit::NSPasteboard;
        let Some(ui) = &self.ui else {
            return;
        };
        let text = ui.settings.convert.input_text();
        let out = ui.settings.convert.ui_options().convert(&text);
        let pb = NSPasteboard::generalPasteboard();
        pb.clearContents();
        let _ = pb.setString_forType(&objc2_foundation::NSString::from_str(&out), unsafe {
            objc2_app_kit::NSPasteboardTypeString
        });
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

    fn close_settings(&mut self) {
        self.end_hotkey_capture();
        if let Some(ui) = &self.ui {
            ui.settings.window().close();
        }
    }

    fn install_tap(&mut self) {
        self.tap = Tap::install();
        if self.tap.is_some() {
            eprintln!("NeoKey keyboard event tap installed");
            self.refresh_ui();
        }
    }

    fn ensure_tap(&mut self) {
        if self.tap.as_ref().is_some_and(Tap::ensure_enabled) {
            self.typing.reset_after_tap_disable();
        }
    }

    fn refresh_input_source(&mut self) {
        self.typing.set_blocked_layout(
            self.typing.engine().settings().vOtherLanguage != 0 && input_source::is_non_english(),
        );
    }

    fn recover_from_failed_relaunch(&mut self) -> bool {
        self.instance_lock = instance::acquire().ok().flatten();
        if self.instance_lock.is_none() {
            return false;
        }
        self.startup_accessibility.recover_after_relaunch_failure();
        self.check_updates_on_startup();
        true
    }

    fn watchdog(&mut self) -> bool {
        if let Some(child) = &mut self.relaunch_child {
            // Check Launch Services' result without blocking the AppKit run loop.
            match child.try_wait() {
                Ok(None) => return false,
                Ok(Some(status)) if status.success() => return true,
                Ok(Some(status)) => {
                    eprintln!("Could not relaunch NeoKey ({status}). Quit and reopen NeoKey.");
                }
                Err(error) => {
                    eprintln!("Could not check NeoKey relaunch ({error}). Quit and reopen NeoKey.");
                }
            }
            self.relaunch_child = None;
            if !self.recover_from_failed_relaunch() {
                return true;
            }
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
                    eprintln!("Restart NeoKey for Spotlight, Alfred and Raycast support");
                    self.install_tap();
                    self.check_updates_on_startup();
                }
                Action::Relaunch => {
                    self.flush_preferences();
                    self.preferences.synchronize();
                    if let Some(path) = &self.bundle_path {
                        // There is no tap yet. Let the replacement take ownership.
                        self.instance_lock.take();
                        match relaunch_bundle(path) {
                            Ok(child) => self.relaunch_child = Some(child),
                            Err(error) => {
                                eprintln!(
                                    "Could not relaunch NeoKey ({error}). Quit and reopen NeoKey."
                                );
                                if !self.recover_from_failed_relaunch() {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        self.refresh_input_source();
        self.flush_preferences();
        self.poll_updates();
        if self
            .ui
            .as_ref()
            .is_some_and(|ui| ui.settings.window().isVisible())
        {
            self.refresh_login();
        }
        if let Some(ui) = &self.ui {
            ui.poll_onboarding(self.tap.is_some());
        }
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
