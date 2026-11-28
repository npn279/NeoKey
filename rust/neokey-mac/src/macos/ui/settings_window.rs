use std::cell::Cell;

use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker, Message};
use objc2_app_kit::{NSApplication, NSButton, NSView, NSWindow, NSWindowStyleMask};
use objc2_foundation::{NSRect, NSSize};

use super::section_about::AboutSection;
use super::section_apps::AppsSection;
use super::section_convert::ConvertSection;
use super::section_general::GeneralSection;
use super::section_macros::MacrosSection;
use super::section_typing::TypingSection;
use super::style::*;
use super::tag;
use super::UiSnapshot;
use crate::macos::status_menu::MenuTarget;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsSection {
    General = 0,
    Typing = 1,
    Macros = 2,
    Apps = 3,
    Convert = 4,
    About = 5,
}

impl SettingsSection {
    pub const ALL: [SettingsSection; 6] = [
        SettingsSection::General,
        SettingsSection::Typing,
        SettingsSection::Macros,
        SettingsSection::Apps,
        SettingsSection::Convert,
        SettingsSection::About,
    ];

    fn title(self) -> &'static str {
        match self {
            Self::General => "Chung",
            Self::Typing => "Kiểu gõ",
            Self::Macros => "Gõ tắt",
            Self::Apps => "Theo ứng dụng",
            Self::Convert => "Chuyển mã",
            Self::About => "Giới thiệu",
        }
    }
}

pub struct SettingsWindow {
    window: Retained<NSWindow>,
    target: Retained<MenuTarget>,
    sidebar_btns: Vec<Retained<NSButton>>,
    pub general: GeneralSection,
    pub typing: TypingSection,
    pub macros: MacrosSection,
    pub apps: AppsSection,
    pub convert: ConvertSection,
    about: AboutSection,
    current: Cell<SettingsSection>,
}

impl SettingsWindow {
    pub fn new(mtm: MainThreadMarker, target: &MenuTarget) -> Self {
        let content_rect = NSRect::new(
            objc2_foundation::NSPoint::new(0.0, 0.0),
            NSSize::new(SETTINGS_WIDTH, SETTINGS_HEIGHT),
        );
        let style = NSWindowStyleMask::Titled
            | NSWindowStyleMask::Closable
            | NSWindowStyleMask::Miniaturizable;
        let window = unsafe { NSWindow::new(mtm) };
        window.setFrame_display(content_rect, true);
        window.setStyleMask(style);
        window.setTitle(&ns("NeoKey"));
        unsafe {
            window.setReleasedWhenClosed(false);
        }

        let sidebar = FlippedView::new(
            mtm,
            NSRect::new(
                objc2_foundation::NSPoint::new(0.0, 0.0),
                NSSize::new(SIDEBAR_WIDTH, SETTINGS_HEIGHT),
            ),
        );
        sidebar.setWantsLayer(true);
        let _sb_bg = rounded_box(mtm, sidebar.bounds(), &sidebar_fill(), 0.0);
        sidebar.addSubview(&_sb_bg);

        let mut sidebar_btns = Vec::new();
        for (i, section) in SettingsSection::ALL.iter().enumerate() {
            let btn = make_quiet_button(mtm, section.title(), tag::SIDEBAR + i as isize);
            place(
                &btn,
                8.0,
                80.0 + i as f64 * 40.0,
                SIDEBAR_WIDTH - 16.0,
                36.0,
            );
            unsafe {
                btn.setTarget(Some(target));
                btn.setAction(Some(sel!(uiAction:)));
            }
            sidebar.addSubview(&btn);
            sidebar_btns.push(btn);
        }

        let content = FlippedView::new(
            mtm,
            NSRect::new(
                objc2_foundation::NSPoint::new(SIDEBAR_WIDTH, 0.0),
                NSSize::new(SETTINGS_WIDTH - SIDEBAR_WIDTH, SETTINGS_HEIGHT),
            ),
        );
        let w = SETTINGS_WIDTH - SIDEBAR_WIDTH;
        let h = SETTINGS_HEIGHT;
        let general = GeneralSection::build(mtm, target, w, h);
        let typing = TypingSection::build(mtm, target, w, h);
        let macros = MacrosSection::build(mtm, target, w, h);
        let apps = AppsSection::build(mtm, target, w, h);
        let convert = ConvertSection::build(mtm, target, w, h);
        let about = AboutSection::build(mtm, target, w, h);

        let root = FlippedView::new(mtm, content_rect);
        root.addSubview(&sidebar);
        root.addSubview(&content);
        for v in [
            general.view(),
            typing.view(),
            macros.view(),
            apps.view(),
            convert.view(),
            about.view(),
        ] {
            content.addSubview(v);
        }
        window.setContentView(Some(&root));

        Self {
            window,
            target: Message::retain(target),
            sidebar_btns,
            general,
            typing,
            macros,
            apps,
            convert,
            about,
            current: Cell::new(SettingsSection::General),
        }
    }

    pub fn show(&self, section: SettingsSection) {
        self.select(section);
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let app = NSApplication::sharedApplication(mtm);
        app.activate();
        self.window.makeKeyAndOrderFront(None);
    }

    fn select(&self, section: SettingsSection) {
        self.current.set(section);
        for (i, btn) in self.sidebar_btns.iter().enumerate() {
            let on = section as usize == i;
            btn.setBordered(!on);
            if on {
                btn.setContentTintColor(Some(&accent_color()));
            }
        }
        let views: [&NSView; 6] = [
            self.general.view(),
            self.typing.view(),
            self.macros.view(),
            self.apps.view(),
            self.convert.view(),
            self.about.view(),
        ];
        for (i, view) in views.iter().enumerate() {
            view.setHidden(i != section as usize);
        }
    }

    pub fn refresh(&self, snap: &UiSnapshot) {
        self.general.refresh(snap);
        self.typing.refresh(snap);
        self.macros.refresh(snap, &self.target);
        self.apps.refresh(snap);
        self.convert.refresh(snap);
        self.about.refresh(snap);
    }
}
