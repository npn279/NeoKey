use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{NSButton, NSSwitch, NSView};

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
        let view = FlippedView::new(
            mtm,
            objc2_foundation::NSRect::new(
                objc2_foundation::NSPoint::new(0.0, 0.0),
                objc2_foundation::NSSize::new(w, h),
            ),
        );
        let title = title_label(mtm, "Chung", 22.0);
        place(&title, 24.0, 16.0, 200.0, 30.0);
        view.addSubview(&title);

        let hotkey_btn = make_button(mtm, "Ghi phím…", tag::HOTKEY_CAPTURE);
        place(&hotkey_btn, w - 220.0, 80.0, 180.0, 40.0);
        let beep_switch = make_switch(mtm, tag::HOTKEY_BEEP, false);
        place(&beep_switch, w - 70.0, 140.0, 44.0, 26.0);
        let step_switch = make_switch(mtm, tag::SETTING + 1, false);
        place(&step_switch, w - 70.0, 220.0, 44.0, 26.0);
        let layout_switch = make_switch(mtm, tag::SETTING + 2, false);
        place(&layout_switch, w - 70.0, 280.0, 44.0, 26.0);
        let chromium_switch = make_switch(mtm, tag::SETTING + 3, false);
        place(&chromium_switch, w - 70.0, 340.0, 44.0, 26.0);

        for (text, y) in [
            ("Phím tắt chuyển ngôn ngữ", 86.0),
            ("Phát âm thanh khi chuyển", 140.0),
            ("Gửi từng phím một", 220.0),
            ("Gõ theo layout phím (AZERTY,…)", 280.0),
            ("Sửa lỗi trình duyệt Chromium", 340.0),
        ] {
            let lab = label(mtm, text, 14.0, false);
            place(&lab, 24.0, y, 420.0, 20.0);
            view.addSubview(&lab);
        }

        for ctrl in [
            &hotkey_btn as &NSView,
            &beep_switch,
            &step_switch,
            &layout_switch,
            &chromium_switch,
        ] {
            view.addSubview(ctrl);
        }
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
        self.hotkey_btn
            .setTitle(&ns("Nhấn tổ hợp phím… (Esc để hủy)"));
    }

    pub fn refresh(&self, snap: &UiSnapshot) {
        self.hotkey_btn.setTitle(&ns(&snap.hotkey.format_labels()));
        set_switch(&self.beep_switch, snap.hotkey.beep);
        set_switch(&self.step_switch, snap.step_by_step);
        set_switch(&self.layout_switch, snap.layout_compat);
        set_switch(&self.chromium_switch, snap.chromium_fix);
    }
}
