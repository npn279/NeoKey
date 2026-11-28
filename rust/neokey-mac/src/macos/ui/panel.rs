//! Right-click control panel: compact, opaque, one accent.
//!
//! Everything heavy lives in the Settings window; the panel keeps only the
//! language toggle, the input type, the spelling switch, and app exit.

use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{
    NSColor, NSPopUpButton, NSPopover, NSPopoverBehavior, NSStatusBarButton, NSSwitch,
    NSViewController,
};
use objc2_foundation::{NSPoint, NSRect, NSRectEdge, NSSize};

use super::convert_map::INPUT_TYPE_LABELS;
use super::style::*;
use super::tag;
use super::UiSnapshot;
use crate::macos::status_menu::MenuTarget;

pub struct Panel {
    popover: Retained<NSPopover>,
    lang_segment: Retained<objc2_app_kit::NSSegmentedControl>,
    subtitle: Retained<objc2_app_kit::NSTextField>,
    input_popup: Retained<NSPopUpButton>,
    spell_switch: Retained<NSSwitch>,
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
        place(&title, 54.0, 17.0, 150.0, 20.0);
        let subtitle = label(mtm, "", 12.0, true);
        place(&subtitle, 54.0, 37.0, 190.0, 16.0);
        let gear = make_icon_button(mtm, "gearshape", "⚙", tag::GEAR);
        place(&gear, PANEL_WIDTH - 48.0, 13.0, 32.0, 32.0);

        // Việt / English, full width.
        let lang_segment = make_lang_segment(mtm, tag::LANG_SEGMENT);
        place(&lang_segment, 16.0, 56.0, PANEL_WIDTH - 32.0, 32.0);

        // Kiểu gõ picker.
        let style_label = label(mtm, "Kiểu gõ", 13.0, false);
        place(&style_label, 16.0, 103.0, 100.0, 18.0);
        let input_popup = NSPopUpButton::new(mtm);
        for name in INPUT_TYPE_LABELS {
            input_popup.addItemWithTitle(&ns(name));
        }
        input_popup.setTag(tag::INPUT_TYPE);
        place(&input_popup, PANEL_WIDTH - 166.0, 97.0, 150.0, 26.0);

        // The single switch row.
        let spell_label = label(mtm, "Kiểm tra chính tả", 13.0, false);
        place(&spell_label, 16.0, 141.0, 200.0, 18.0);
        let spell_switch = make_switch(mtm, tag::PANEL_SPELL, true);
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
        place(&settings_btn, 16.0, 185.0, 120.0, 28.0);
        settings_btn.setContentTintColor(Some(&NSColor::secondaryLabelColor()));
        let quit_btn = make_quiet_button(mtm, "Thoát", tag::QUIT);
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
            &style_label,
            &input_popup,
            &spell_label,
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
        }
    }

    pub fn show(&self, button: &NSStatusBarButton) {
        if self.popover.isShown() {
            self.popover.close();
            return;
        }
        // Anchor to the whole button: a rect outside its bounds makes AppKit refuse to show.
        let bounds = button.bounds();
        if let Some(mtm) = MainThreadMarker::new() {
            objc2_app_kit::NSApplication::sharedApplication(mtm).activate();
        }
        self.popover
            .showRelativeToRect_ofView_preferredEdge(bounds, button, NSRectEdge::MinY);
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
        self.input_popup
            .selectItemAtIndex(snap.settings.vInputType.clamp(0, 3) as isize);
        set_switch(&self.spell_switch, snap.spelling);
    }

    /// Debug snapshot support: the popover's root view (`ui::snapshot`).
    pub(crate) fn snapshot_view(&self) -> Option<Retained<objc2_app_kit::NSView>> {
        self.popover
            .contentViewController()
            .map(|controller| controller.view())
    }
}
