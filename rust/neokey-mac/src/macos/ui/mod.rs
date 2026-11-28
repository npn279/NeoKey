//! Native AppKit settings UI for NeoKey.
//!
//! Omitted from the design mockups (no UI yet): FreeMark (`vFreeMark`), status-bar icon
//! style variants, launch-at-login, and the About-tab update checker.

mod convert_map;
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
mod style;

pub use hotkey::Hotkey;
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
    pub focus_app: Option<String>,
    #[allow(dead_code)]
    pub focus_launcher: bool,
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

    pub fn refresh(&self, snapshot: &UiSnapshot) {
        self.panel.refresh(snapshot);
        self.settings.refresh(snapshot);
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
    pub const GEAR: isize = 1;
    pub const LANG_VI: isize = 2;
    pub const LANG_EN: isize = 3;
    pub const INPUT_TYPE: isize = 10;
    pub const CODE_TABLE: isize = 20;
    pub const PANEL_SPELL: isize = 21;
    pub const PANEL_SMART: isize = 22;
    pub const OPEN_MACROS: isize = 30;
    pub const OPEN_CONVERT: isize = 31;
    pub const QUIT: isize = 32;
    pub const SIDEBAR: isize = 100;
    pub const HOTKEY_CAPTURE: isize = 110;
    pub const HOTKEY_BEEP: isize = 111;
    pub const SETTING: isize = 200;
    pub const MACRO_ADD: isize = 300;
    pub const MACRO_DELETE: isize = 301;
    pub const MACRO_IMPORT: isize = 302;
    pub const MACRO_EXPORT: isize = 303;
    pub const MACRO_ROW_DELETE: isize = 310;
    pub const APP_LANG: isize = 400;
    pub const CONVERT_RUN: isize = 500;
    pub const CONVERT_PASTE: isize = 501;
    pub const CONVERT_COPY: isize = 502;
    pub const CONVERT_SWAP: isize = 503;
    pub const LINK: isize = 600;
    pub const ONBOARDING_OPEN: isize = 700;
    pub const ONBOARDING_RETRY: isize = 701;
    pub const ONBOARDING_QUIT: isize = 702;
}

pub type UiHandle = Rc<Ui>;
