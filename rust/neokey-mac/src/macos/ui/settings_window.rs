//! Settings window: sidebar navigation + grouped card sections.
//!
//! The selected sidebar item is a real selection: an accent-tinted pill behind
//! the item plus accent icon/text (unselected items reset to secondary icon +
//! plain label), so the state never depends on removing a button border.

use std::cell::Cell;

use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker, Message};
use objc2_app_kit::{NSBox, NSColor, NSView, NSWindow, NSWindowStyleMask};
use objc2_foundation::{NSPoint, NSRect, NSSize};

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

const ITEM_TOP: f64 = 76.0;
const ITEM_HEIGHT: f64 = 32.0;
const ITEM_PITCH: f64 = 38.0;
const ITEM_INSET: f64 = 8.0;

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

    fn symbol(self) -> &'static str {
        match self {
            Self::General => "gearshape",
            Self::Typing => "keyboard",
            Self::Macros => "square.and.pencil",
            Self::Apps => "rectangle.on.rectangle",
            Self::Convert => "arrow.left.arrow.right",
            Self::About => "info.circle",
        }
    }

    /// ASCII slug used for snapshot file names.
    pub(crate) fn slug(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Typing => "typing",
            Self::Macros => "macros",
            Self::Apps => "apps",
            Self::Convert => "convert",
            Self::About => "about",
        }
    }
}

pub struct SettingsWindow {
    window: Retained<NSWindow>,
    target: Retained<MenuTarget>,
    sidebar_items: Vec<SidebarItem>,
    pill: Retained<NSBox>,
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
            NSPoint::new(0.0, 0.0),
            NSSize::new(SETTINGS_WIDTH, SETTINGS_HEIGHT),
        );
        let style = NSWindowStyleMask::Titled
            | NSWindowStyleMask::Closable
            | NSWindowStyleMask::Miniaturizable;
        let window = unsafe { NSWindow::new(mtm) };
        window.setFrame_display(content_rect, true);
        window.setStyleMask(style);
        window.setTitle(&ns("Cài đặt NeoKey"));
        unsafe {
            window.setReleasedWhenClosed(false);
        }

        // Sidebar: a subtly different background from the content area.
        let sidebar = FlippedView::new(
            mtm,
            NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(SIDEBAR_WIDTH, SETTINGS_HEIGHT),
            ),
        );
        let background = rounded_box(
            mtm,
            sidebar.bounds(),
            &NSColor::underPageBackgroundColor(),
            0.0,
        );
        sidebar.addSubview(&background);

        let icon = app_icon_view(mtm, 28.0);
        place(&icon, 16.0, 18.0, 28.0, 28.0);
        let name = title_label(mtm, "NeoKey", 15.0);
        place(&name, 52.0, 20.0, 130.0, 20.0);
        let tagline = label(mtm, "Bộ gõ tiếng Việt", 12.0, true);
        place(&tagline, 52.0, 40.0, 140.0, 16.0);
        for view in [&icon as &NSView, &name, &tagline] {
            sidebar.addSubview(view);
        }

        // Selection pill spans the item with 8 pt side insets.
        let item_w = SIDEBAR_WIDTH - 2.0 * ITEM_INSET;
        let pill = rounded_box(
            mtm,
            NSRect::new(
                NSPoint::new(ITEM_INSET, ITEM_TOP),
                NSSize::new(item_w, ITEM_HEIGHT),
            ),
            &soft_accent_fill(false),
            8.0,
        );
        sidebar.addSubview(&pill);

        let mut sidebar_items = Vec::new();
        for (i, section) in SettingsSection::ALL.iter().enumerate() {
            let item = SidebarItem::new(
                mtm,
                section.title(),
                section.symbol(),
                tag::SIDEBAR + i as isize,
                item_w,
            );
            place(
                &item.view,
                ITEM_INSET,
                ITEM_TOP + i as f64 * ITEM_PITCH,
                item_w,
                ITEM_HEIGHT,
            );
            unsafe {
                item.button.setTarget(Some(target));
                item.button.setAction(Some(sel!(uiAction:)));
            }
            sidebar.addSubview(&item.view);
            sidebar_items.push(item);
        }

        // Content area: windowBackgroundColor so the white cards stand out.
        let content = FlippedView::new(
            mtm,
            NSRect::new(
                NSPoint::new(SIDEBAR_WIDTH, 0.0),
                NSSize::new(SETTINGS_WIDTH - SIDEBAR_WIDTH, SETTINGS_HEIGHT),
            ),
        );
        let content_bg = rounded_box(
            mtm,
            content.bounds(),
            &NSColor::windowBackgroundColor(),
            0.0,
        );
        content.addSubview(&content_bg);

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
        let divider = separator_box(
            mtm,
            NSRect::new(
                NSPoint::new(SIDEBAR_WIDTH - 1.0, 0.0),
                NSSize::new(1.0, SETTINGS_HEIGHT),
            ),
        );
        root.addSubview(&divider);
        for view in [
            general.view(),
            typing.view(),
            macros.view(),
            apps.view(),
            convert.view(),
            about.view(),
        ] {
            content.addSubview(view);
        }
        window.setContentView(Some(&root));

        Self {
            window,
            target: Message::retain(target),
            sidebar_items,
            pill,
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
        let app = objc2_app_kit::NSApplication::sharedApplication(mtm);
        app.activate();
        self.window.makeKeyAndOrderFront(None);
    }

    pub(crate) fn select(&self, section: SettingsSection) {
        self.current.set(section);
        let index = section as usize;
        self.pill.setFrameOrigin(NSPoint::new(
            ITEM_INSET,
            ITEM_TOP + index as f64 * ITEM_PITCH,
        ));
        self.retint(app_is_dark());
        let views: [&NSView; 6] = [
            self.general.view(),
            self.typing.view(),
            self.macros.view(),
            self.apps.view(),
            self.convert.view(),
            self.about.view(),
        ];
        for (i, view) in views.iter().enumerate() {
            view.setHidden(i != index);
        }
    }

    /// Selected item: accent pill + accent icon/text; the rest reset to a
    /// secondary icon + plain label, so there is no stale tint state.
    fn retint(&self, dark: bool) {
        let accent = accent_color(dark);
        self.pill.setFillColor(&soft_accent_fill(dark));
        for (i, item) in self.sidebar_items.iter().enumerate() {
            item.set_selected(i == self.current.get() as usize, &accent);
        }
    }

    pub fn refresh(&self, snap: &UiSnapshot, dark: bool) {
        self.retint(dark);
        self.general.refresh(snap);
        self.typing.refresh(snap, dark);
        self.macros.refresh(snap, &self.target);
        self.apps.refresh(snap);
        self.convert.refresh(snap);
        self.about.refresh(dark);
    }

    /// Debug snapshot support: the window's content view (`ui::snapshot`).
    pub(crate) fn snapshot_view(&self) -> Option<Retained<NSView>> {
        self.window.contentView()
    }
}
