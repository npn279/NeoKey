use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{NSButton, NSSwitch, NSView};
use objc2_foundation::{NSPoint, NSRect, NSSize};

use super::style::*;
use super::tag;
use super::UiSnapshot;
use crate::macos::status_menu::MenuTarget;

pub struct GeneralSection {
    view: Retained<FlippedView>,
    hotkey_btn: Retained<NSButton>,
    beep_switch: Retained<NSSwitch>,
    step_switch: Retained<NSSwitch>,
    layout_switch: Retained<NSSwitch>,
    chromium_switch: Retained<NSSwitch>,
}

impl GeneralSection {
    pub fn build(mtm: MainThreadMarker, target: &MenuTarget, w: f64, h: f64) -> Self {
        let view = FlippedView::new(mtm, NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(w, h)));
        let title = section_title(mtm, "Chung");
        place(&title, MARGIN, 18.0, w - 2.0 * MARGIN, 26.0);
        view.addSubview(&title);

        let card_w = w - 2.0 * MARGIN;

        // Phím tắt.
        let hotkey_header = section_header(mtm, "Phím tắt");
        place(&hotkey_header, MARGIN, 58.0, 300.0, 16.0);
        view.addSubview(&hotkey_header);
        let hotkey_card = card(mtm, MARGIN, 80.0, card_w, 2.0 * ROW_HEIGHT + 12.0);
        let hotkey_rect = row(
            mtm,
            &hotkey_card,
            6.0,
            "Phím tắt chuyển ngôn ngữ",
            200.0,
            30.0,
        );
        let hotkey_btn = make_button(mtm, "Ghi phím…", tag::HOTKEY_CAPTURE);
        add_control(&hotkey_card, &hotkey_btn, hotkey_rect);
        row_separator(mtm, &hotkey_card, ROW_HEIGHT + 6.0);
        let beep_rect = row(
            mtm,
            &hotkey_card,
            ROW_HEIGHT + 6.0,
            "Phát âm thanh khi chuyển",
            switch_w(mtm),
            switch_h(mtm),
        );
        let beep_switch = make_switch(mtm, tag::HOTKEY_BEEP, false);
        add_control(&hotkey_card, &beep_switch, beep_rect);
        view.addSubview(&hotkey_card);

        // Nâng cao.
        let advanced_top = 80.0 + 2.0 * ROW_HEIGHT + 12.0 + 18.0;
        let advanced_header = section_header(mtm, "Nâng cao");
        place(&advanced_header, MARGIN, advanced_top, 300.0, 16.0);
        view.addSubview(&advanced_header);
        let advanced_card = card(
            mtm,
            MARGIN,
            advanced_top + 22.0,
            card_w,
            3.0 * ROW_HEIGHT + 12.0,
        );
        let step_rect = row(
            mtm,
            &advanced_card,
            6.0,
            "Gửi từng phím một",
            switch_w(mtm),
            switch_h(mtm),
        );
        let step_switch = make_switch(mtm, tag::SETTING + 1, false);
        add_control(&advanced_card, &step_switch, step_rect);
        row_separator(mtm, &advanced_card, ROW_HEIGHT + 6.0);
        let layout_rect = row(
            mtm,
            &advanced_card,
            ROW_HEIGHT + 6.0,
            "Gõ theo layout phím (AZERTY,…)",
            switch_w(mtm),
            switch_h(mtm),
        );
        let layout_switch = make_switch(mtm, tag::SETTING + 2, false);
        add_control(&advanced_card, &layout_switch, layout_rect);
        row_separator(mtm, &advanced_card, 2.0 * ROW_HEIGHT + 6.0);
        let chromium_rect = row(
            mtm,
            &advanced_card,
            2.0 * ROW_HEIGHT + 6.0,
            "Sửa lỗi trình duyệt Chromium",
            switch_w(mtm),
            switch_h(mtm),
        );
        let chromium_switch = make_switch(mtm, tag::SETTING + 3, false);
        add_control(&advanced_card, &chromium_switch, chromium_rect);
        view.addSubview(&advanced_card);

        unsafe {
            hotkey_btn.setTarget(Some(target));
            hotkey_btn.setAction(Some(sel!(uiAction:)));
            beep_switch.setTarget(Some(target));
            beep_switch.setAction(Some(sel!(uiAction:)));
            step_switch.setTarget(Some(target));
            step_switch.setAction(Some(sel!(uiAction:)));
            layout_switch.setTarget(Some(target));
            layout_switch.setAction(Some(sel!(uiAction:)));
            chromium_switch.setTarget(Some(target));
            chromium_switch.setAction(Some(sel!(uiAction:)));
        }

        Self {
            view,
            hotkey_btn,
            beep_switch,
            step_switch,
            layout_switch,
            chromium_switch,
        }
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }

    pub fn set_capturing(&self) {
        self.hotkey_btn.setTitle(&ns("Nhấn tổ hợp… (Esc huỷ)"));
    }

    pub fn refresh(&self, snap: &UiSnapshot) {
        // Compact symbols (⌃⌥⌘⇧…) never truncate, unlike the spaced labels;
        // the full spaced chord stays available as a tooltip.
        self.hotkey_btn.setTitle(&ns(&snap.hotkey.format_symbols()));
        self.hotkey_btn
            .setToolTip(Some(&ns(&snap.hotkey.format_labels())));
        set_switch(&self.beep_switch, snap.hotkey.beep);
        set_switch(&self.step_switch, snap.step_by_step);
        set_switch(&self.layout_switch, snap.layout_compat);
        set_switch(&self.chromium_switch, snap.chromium_fix);
    }
}
