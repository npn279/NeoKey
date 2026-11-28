//! Right-click control panel: compact, opaque, one accent.
//!
//! Everything heavy lives in the Settings window; the panel keeps only the
//! language toggle, the input type, the spelling switch, and app exit.

use std::cell::RefCell;
use std::ptr::NonNull;
use std::rc::Rc;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{
    NSColor, NSEvent, NSEventMask, NSPopUpButton, NSPopover, NSPopoverBehavior, NSStatusBarButton,
    NSSwitch, NSViewController,
};
use objc2_foundation::{NSPoint, NSRect, NSRectEdge, NSSize};

use super::convert_map::{input_type_help, INPUT_TYPE_LABELS, INPUT_TYPE_TOOLTIPS};
use super::help::add_help;
use super::style::*;
use super::tag;
use super::UiSnapshot;
use crate::macos::status_menu::MenuTarget;

const SPELL_TOOLTIP: &str = "Bật: chỉ thêm dấu khi từ đang gõ là tiếng Việt hợp lệ, \
     nên gõ tiếng Anh hay tên riêng ít bị biến chữ. Tắt: luôn thêm dấu theo phím gõ.";

pub struct Panel {
    popover: Retained<NSPopover>,
    lang_segment: Retained<objc2_app_kit::NSSegmentedControl>,
    subtitle: Retained<objc2_app_kit::NSTextField>,
    input_popup: Retained<NSPopUpButton>,
    spell_switch: Retained<NSSwitch>,
    /// Global mouse-down monitor installed while the panel is open. A
    /// Transient popover only closes when NeoKey resigns active, but macOS 14+
    /// may refuse to activate a menu-bar app, so clicks in other apps would
    /// otherwise leave the panel open.
    outside_click: Rc<RefCell<Option<Retained<AnyObject>>>>,
}

impl Panel {
    pub fn new(mtm: MainThreadMarker, target: &MenuTarget) -> Self {
        let popover = NSPopover::new(mtm);
        popover.setBehavior(NSPopoverBehavior::Transient);

        let root = SolidView::new(
            mtm,
            NSRect::new(
                NSPoint::new(0.0, 0.0),
                NSSize::new(PANEL_WIDTH, PANEL_HEIGHT),
            ),
        );

        // Header: icon, name, switch hotkey, settings gear.
        let icon = app_icon_view(mtm, 28.0);
        place(&icon, 16.0, 15.0, 28.0, 28.0);
        let title = title_label(mtm, "NeoKey", 15.0);
        title.setToolTip(Some(&ns("NeoKey — bộ gõ tiếng Việt")));
        icon.setToolTip(Some(&ns("NeoKey — bộ gõ tiếng Việt")));
        place(&title, 54.0, 17.0, 150.0, 20.0);
        let subtitle = label(mtm, "", 12.0, true);
        place(&subtitle, 54.0, 37.0, 190.0, 16.0);
        let gear = make_icon_button(mtm, "gearshape", "⚙", tag::GEAR);
        gear.setToolTip(Some(&ns("Mở cửa sổ Cài đặt")));
        place(&gear, PANEL_WIDTH - 48.0, 13.0, 32.0, 32.0);

        // Việt / English, full width.
        let lang_segment = make_lang_segment(mtm, tag::LANG_SEGMENT);
        place(&lang_segment, 16.0, 56.0, PANEL_WIDTH - 32.0, 32.0);

        // Kiểu gõ picker.
        let style_label = label(mtm, "Kiểu gõ", 13.0, false);
        let style_help = input_type_help();
        style_label.setToolTip(Some(&ns(&style_help)));
        place(&style_label, 16.0, 103.0, 100.0, 18.0);
        root.addSubview(&style_label);
        add_help(mtm, &root, &style_label, &style_help);
        let input_popup = NSPopUpButton::new(mtm);
        for (i, name) in INPUT_TYPE_LABELS.iter().enumerate() {
            input_popup.addItemWithTitle(&ns(name));
            if let Some(item) = input_popup.lastItem() {
                item.setToolTip(Some(&ns(INPUT_TYPE_TOOLTIPS[i])));
            }
        }
        input_popup.setTag(tag::INPUT_TYPE);
        place(&input_popup, PANEL_WIDTH - 166.0, 97.0, 150.0, 26.0);

        // The single switch row.
        let spell_label = label(mtm, "Kiểm tra chính tả", 13.0, false);
        spell_label.setToolTip(Some(&ns(SPELL_TOOLTIP)));
        place(&spell_label, 16.0, 141.0, 200.0, 18.0);
        root.addSubview(&spell_label);
        add_help(mtm, &root, &spell_label, SPELL_TOOLTIP);
        let spell_switch = make_switch(mtm, tag::PANEL_SPELL, true);
        spell_switch.setToolTip(Some(&ns(SPELL_TOOLTIP)));
        place(
            &spell_switch,
            PANEL_WIDTH - 16.0 - switch_w(mtm),
            150.0 - switch_h(mtm) / 2.0,
            switch_w(mtm),
            switch_h(mtm),
        );

        // Footer.
        let separator = separator_box(
            mtm,
            NSRect::new(
                NSPoint::new(16.0, 177.0),
                NSSize::new(PANEL_WIDTH - 32.0, 1.0),
            ),
        );
        let settings_btn = make_quiet_button(mtm, "Cài đặt…", tag::GEAR);
        settings_btn.setToolTip(Some(&ns(
            "Phím tắt, bảng mã, gõ tắt, ngôn ngữ theo từng ứng dụng, chuyển mã…",
        )));
        place(&settings_btn, 16.0, 185.0, 120.0, 28.0);
        settings_btn.setContentTintColor(Some(&NSColor::secondaryLabelColor()));
        let quit_btn = make_quiet_button(mtm, "Thoát", tag::QUIT);
        quit_btn.setToolTip(Some(&ns("Tắt NeoKey; bàn phím trở lại gõ bình thường")));
        place(&quit_btn, PANEL_WIDTH - 86.0, 185.0, 70.0, 28.0);
        quit_btn.setContentTintColor(Some(&NSColor::secondaryLabelColor()));

        unsafe {
            gear.setTarget(Some(target));
            gear.setAction(Some(sel!(uiAction:)));
            settings_btn.setTarget(Some(target));
            settings_btn.setAction(Some(sel!(uiAction:)));
            quit_btn.setTarget(Some(target));
            quit_btn.setAction(Some(sel!(uiAction:)));
            lang_segment.setTarget(Some(target));
            lang_segment.setAction(Some(sel!(uiAction:)));
            input_popup.setTarget(Some(target));
            input_popup.setAction(Some(sel!(uiAction:)));
            spell_switch.setTarget(Some(target));
            spell_switch.setAction(Some(sel!(uiAction:)));
        }

        for view in [
            &icon as &objc2_app_kit::NSView,
            &title,
            &subtitle,
            &gear,
            &lang_segment,
            &input_popup,
            &spell_switch,
            &separator,
            &settings_btn,
            &quit_btn,
        ] {
            root.addSubview(view);
        }

        let controller = NSViewController::new(mtm);
        controller.setView(&root);
        popover.setContentViewController(Some(&controller));
        popover.setContentSize(NSSize::new(PANEL_WIDTH, PANEL_HEIGHT));

        Self {
            popover,
            lang_segment,
            subtitle,
            input_popup,
            spell_switch,
            outside_click: Rc::new(RefCell::new(None)),
        }
    }

    pub fn show(&self, button: &NSStatusBarButton) {
        if self.popover.isShown() {
            self.popover.close();
            remove_monitor(&self.outside_click);
            return;
        }
        // Anchor to the whole button: a rect outside its bounds makes AppKit refuse to show.
        let bounds = button.bounds();
        if let Some(mtm) = MainThreadMarker::new() {
            objc2_app_kit::NSApplication::sharedApplication(mtm).activate();
        }
        self.popover
            .showRelativeToRect_ofView_preferredEdge(bounds, button, NSRectEdge::MinY);
        self.watch_outside_clicks();
    }

    /// Closes the panel on the next mouse-down in another app. Global monitors
    /// never see clicks in NeoKey itself, so the status item still toggles.
    fn watch_outside_clicks(&self) {
        remove_monitor(&self.outside_click);
        let popover = self.popover.clone();
        let slot = Rc::clone(&self.outside_click);
        let handler = RcBlock::new(move |_event: NonNull<NSEvent>| {
            popover.close();
            remove_monitor(&slot);
        });
        let monitor = NSEvent::addGlobalMonitorForEventsMatchingMask_handler(
            NSEventMask::LeftMouseDown | NSEventMask::RightMouseDown | NSEventMask::OtherMouseDown,
            &handler,
        );
        *self.outside_click.borrow_mut() = monitor;
    }

    pub fn refresh(&self, snap: &UiSnapshot, dark: bool) {
        let accent = accent_color(dark);
        let vi = snap.settings.vLanguage != 0;
        self.lang_segment.setSelectedSegment(if vi { 0 } else { 1 });
        self.lang_segment
            .setSelectedSegmentBezelColor(Some(&accent));
        let symbols = snap.hotkey.format_symbols();
        self.subtitle
            .setStringValue(&ns(&format!("{symbols} để chuyển")));
        let labels = snap.hotkey.format_labels();
        self.subtitle.setToolTip(Some(&ns(&format!(
            "Nhấn {labels} ở bất kỳ đâu để chuyển Việt ↔ English. Đổi phím trong Cài đặt → Chung."
        ))));
        self.lang_segment
            .setToolTip_forSegment(Some(&ns(&format!("Gõ tiếng Việt có dấu ({labels})"))), 0);
        self.lang_segment.setToolTip_forSegment(
            Some(&ns(&format!(
                "Gõ tiếng Anh, NeoKey không thêm dấu ({labels})"
            ))),
            1,
        );
        let input_type = snap.settings.vInputType.clamp(0, 3) as usize;
        self.input_popup.selectItemAtIndex(input_type as isize);
        // Item tooltips only show while the menu is open; the closed button
        // describes the selected type.
        self.input_popup
            .setToolTip(Some(&ns(INPUT_TYPE_TOOLTIPS[input_type])));
        set_switch(&self.spell_switch, snap.spelling);
    }

    /// Debug snapshot support: the popover's root view (`ui::snapshot`).
    pub(crate) fn snapshot_view(&self) -> Option<Retained<objc2_app_kit::NSView>> {
        self.popover
            .contentViewController()
            .map(|controller| controller.view())
    }
}

fn remove_monitor(slot: &RefCell<Option<Retained<AnyObject>>>) {
    if let Some(monitor) = slot.borrow_mut().take() {
        // The monitor came from addGlobalMonitorForEventsMatchingMask:handler:.
        unsafe { NSEvent::removeMonitor(&monitor) };
    }
}
