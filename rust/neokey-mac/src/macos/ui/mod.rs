//! Native AppKit settings UI for NeoKey.
//! Includes the menu-bar panel, settings, clipboard tools and Accessibility onboarding.

pub(super) mod alert;
mod convert_map;
pub(super) use convert_map::{CaseMode, ConvertUiOptions};
mod help;
mod hotkey;
mod onboarding;
mod panel;
mod section_about;
mod section_apps;
mod section_convert;
mod section_general;
mod section_macros;
mod section_typing;
mod settings_window;
pub(crate) mod snapshot;
pub(crate) mod style;

pub use hotkey::{Capture, Hotkey, HotkeyRecorder};
pub use panel::Panel;
pub use settings_window::{SettingsSection, SettingsWindow};

use neokey_engine::Settings;
use std::rc::Rc;

use objc2::MainThreadMarker;

use super::status_menu::MenuTarget;

/// Values the UI mirrors from [`super::State`].
#[derive(Clone)]
pub struct UiSnapshot {
    pub settings: Settings,
    pub spelling: bool,
    pub step_by_step: bool,
    pub layout_compat: bool,
    pub chromium_fix: bool,
    pub hotkey: Hotkey,
    pub convert_hotkey: Hotkey,
    pub convert_alert: bool,
    pub login_status: super::login_item::LoginStatus,
    pub gray_icon: bool,
    pub dock_icon: bool,
    pub show_on_startup: bool,
    pub update_on_startup: bool,
    pub checking_updates: bool,
    pub tap_ready: bool,
    pub macros: Vec<(String, String)>,
}

pub struct Ui {
    pub panel: Panel,
    pub settings: SettingsWindow,
    pub(crate) onboarding: onboarding::Onboarding,
}

impl Ui {
    pub fn new(mtm: MainThreadMarker, target: &MenuTarget) -> Self {
        Self {
            panel: Panel::new(mtm, target),
            settings: SettingsWindow::new(mtm, target),
            onboarding: onboarding::Onboarding::new(mtm, target),
        }
    }

    pub fn refresh(&self, snapshot: &UiSnapshot, dark: bool) {
        self.panel.refresh(snapshot, dark);
        self.settings.refresh(snapshot, dark);
    }

    pub fn show_settings(&self, section: SettingsSection) {
        self.settings.show(section);
    }

    pub fn show_panel(&self, button: &objc2_app_kit::NSStatusBarButton) {
        self.panel.show(button);
    }

    pub fn show_onboarding_if_needed(&self, trusted: bool) {
        if !trusted {
            self.onboarding.show();
        }
    }

    pub fn poll_onboarding(&self, trusted: bool) {
        self.onboarding.poll(trusted);
    }
}

/// Action tags shared by panel and settings (`MenuTarget::uiAction:`).
pub mod tag {
    // Panel.
    /// Gear icon and "Cài đặt…" footer button (both open Settings).
    pub const GEAR: isize = 1;
    /// Panel Việt / English segmented control; segment 0 is Việt.
    pub const LANG_SEGMENT: isize = 2;
    /// Panel spelling switch.
    pub const PANEL_SPELL: isize = 21;
    pub const QUIT: isize = 32;

    // Panel and Settings.
    /// Input type: panel `NSPopUpButton` and settings `NSSegmentedControl`
    /// both send this; the handler reads either `selectedSegment` or the
    /// popup's selected index as the mode.
    pub const INPUT_TYPE: isize = 10;
    /// Settings code-table popup.
    pub const CODE_TABLE: isize = 20;
    /// Settings "Nhớ ngôn ngữ theo ứng dụng" switch.
    pub const SMART_SWITCH: isize = 22;

    // Settings.
    pub const SIDEBAR: isize = 100;
    pub const HOTKEY_CAPTURE: isize = 110;
    pub const HOTKEY_BEEP: isize = 111;
    pub const SETTING: isize = 200;
    pub const MACRO_ADD: isize = 300;
    pub const MACRO_IMPORT: isize = 302;
    pub const MACRO_EXPORT: isize = 303;
    pub const MACRO_ROW_DELETE: isize = 10_000;
    /// Per-app rows in Settings → Theo ứng dụng; the row index is the tag
    /// offset, read through `selectedSegment` (0 = Việt).
    pub const APP_LANG: isize = 12_000;
    pub const CONVERT_RUN: isize = 500;
    pub const CONVERT_PASTE: isize = 501;
    pub const CONVERT_COPY: isize = 502;
    pub const CONVERT_SWAP: isize = 503;
    pub const CONVERT_OPTIONS: isize = 504;
    pub const CONVERT_HOTKEY: isize = 505;
    pub const CONVERT_ALERT: isize = 506;
    pub const CONVERT_QUICK: isize = 507;
    pub const CONVERT_CLEAR_HOTKEY: isize = 508;
    pub const LINK: isize = 600;
    pub const UPDATE: isize = 610;
    pub const STARTUP_UPDATE: isize = 611;
    pub const LOGIN: isize = 620;
    pub const GRAY_ICON: isize = 621;
    pub const DOCK_ICON: isize = 622;
    pub const STARTUP_UI: isize = 623;
    pub const RESET: isize = 624;
    pub const CLOSE_SETTINGS: isize = 625;
    pub const OPEN_LOGIN_SETTINGS: isize = 626;
    pub const OPEN_CONVERT: isize = 627;
    pub const OPEN_MACROS: isize = 628;
    pub const OPEN_ABOUT: isize = 629;
    pub const ONBOARDING_OPEN: isize = 700;
    pub const ONBOARDING_RETRY: isize = 701;
    pub const ONBOARDING_QUIT: isize = 702;
}

pub type UiHandle = Rc<Ui>;
