use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{
    NSButton, NSPopUpButton, NSPopover, NSPopoverBehavior, NSStatusBarButton, NSSwitch, NSView,
    NSViewController,
};
use objc2_foundation::{NSRect, NSRectEdge};

use super::convert_map::{input_type_hint, CODE_TABLE_LABELS, INPUT_TYPE_LABELS};
use super::style::*;
use super::tag;
use super::UiSnapshot;
use crate::macos::status_menu::MenuTarget;

pub struct Panel {
    popover: Retained<NSPopover>,
    vi_btn: Retained<NSButton>,
    en_btn: Retained<NSButton>,
    hotkey_hint: Retained<objc2_app_kit::NSTextField>,
    input_btns: [Retained<NSButton>; 4],
    hint: Retained<objc2_app_kit::NSTextField>,
    code_popup: Retained<NSPopUpButton>,
    spell_switch: Retained<NSSwitch>,
    smart_switch: Retained<NSSwitch>,
    smart_banner: Retained<objc2_app_kit::NSTextField>,
}

impl Panel {
    pub fn new(mtm: MainThreadMarker, target: &MenuTarget) -> Self {
        let popover = NSPopover::new(mtm);
        popover.setBehavior(NSPopoverBehavior::Transient);

        let root = FlippedView::new(
            mtm,
            NSRect::new(
                objc2_foundation::NSPoint::new(0.0, 0.0),
                objc2_foundation::NSSize::new(PANEL_WIDTH, PANEL_HEIGHT),
            ),
        );
        root.setWantsLayer(true);

        let gear = make_quiet_button(mtm, "⚙", tag::GEAR);
        place(&gear, PANEL_WIDTH - 52.0, 16.0, 36.0, 36.0);
        unsafe {
            gear.setTarget(Some(target));
            gear.setAction(Some(sel!(uiAction:)));
        }

        let title = title_label(mtm, "NeoKey", 15.0);
        place(&title, 58.0, 20.0, 120.0, 22.0);
        let subtitle = label(mtm, "Bộ gõ tiếng Việt", 12.0, true);
        place(&subtitle, 58.0, 40.0, 200.0, 18.0);
        let icon = app_icon_view(mtm, 30.0);
        place(&icon, 20.0, 20.0, 30.0, 30.0);

        let vi_btn = make_button(mtm, "Việt", tag::LANG_VI);
        let en_btn = make_button(mtm, "English", tag::LANG_EN);
        place(&vi_btn, 26.0, 90.0, 158.0, 76.0);
        place(&en_btn, 196.0, 90.0, 158.0, 76.0);
        for btn in [&vi_btn, &en_btn] {
            unsafe {
                btn.setTarget(Some(target));
                btn.setAction(Some(sel!(uiAction:)));
            }
        }

        let hotkey_hint = label(mtm, "", 12.0, true);
        place(&hotkey_hint, 20.0, 178.0, PANEL_WIDTH - 40.0, 18.0);

        let header = section_header(mtm, "KIỂU GÕ");
        place(&header, 20.0, 210.0, 200.0, 16.0);

        let input_btns = [
            make_button(mtm, INPUT_TYPE_LABELS[0], tag::INPUT_TYPE),
            make_button(mtm, INPUT_TYPE_LABELS[1], tag::INPUT_TYPE + 1),
            make_button(mtm, INPUT_TYPE_LABELS[2], tag::INPUT_TYPE + 2),
            make_button(mtm, INPUT_TYPE_LABELS[3], tag::INPUT_TYPE + 3),
        ];
        for (i, btn) in input_btns.iter().enumerate() {
            place(btn, 20.0 + i as f64 * 88.0, 232.0, 84.0, 44.0);
            unsafe {
                btn.setTarget(Some(target));
                btn.setAction(Some(sel!(uiAction:)));
            }
        }

        let hint = label(mtm, "", 12.0, true);
        place(&hint, 20.0, 282.0, PANEL_WIDTH - 40.0, 32.0);

        let code_popup = NSPopUpButton::new(mtm);
        for title in CODE_TABLE_LABELS {
            code_popup.addItemWithTitle(&ns(title));
        }
        code_popup.setTag(tag::CODE_TABLE);
        place(&code_popup, 20.0, 330.0, PANEL_WIDTH - 40.0, 28.0);
        unsafe {
            code_popup.setTarget(Some(target));
            code_popup.setAction(Some(sel!(uiAction:)));
        }

        let spell_switch = make_switch(mtm, tag::PANEL_SPELL, true);
        place(&spell_switch, PANEL_WIDTH - 70.0, 378.0, 44.0, 26.0);
        let spell_l = label(mtm, "Kiểm tra chính tả", 14.0, false);
        place(&spell_l, 20.0, 376.0, 240.0, 20.0);
        let smart_switch = make_switch(mtm, tag::PANEL_SMART, true);
        place(&smart_switch, PANEL_WIDTH - 70.0, 432.0, 44.0, 26.0);
        let smart_l = label(mtm, "Nhớ theo ứng dụng", 14.0, false);
        place(&smart_l, 20.0, 430.0, 240.0, 20.0);
        for sw in [&spell_switch, &smart_switch] {
            unsafe {
                sw.setTarget(Some(target));
                sw.setAction(Some(sel!(uiAction:)));
            }
        }

        let banner_box = rounded_box(
            mtm,
            NSRect::new(
                objc2_foundation::NSPoint::new(20.0, 478.0),
                objc2_foundation::NSSize::new(PANEL_WIDTH - 40.0, 54.0),
            ),
            &soft_accent_fill(),
            10.0,
        );
        let smart_banner = label(mtm, "", 12.0, false);
        place(&smart_banner, 36.0, 490.0, PANEL_WIDTH - 72.0, 36.0);

        let macro_btn = make_quiet_button(mtm, "Gõ tắt…", tag::OPEN_MACROS);
        let convert_btn = make_quiet_button(mtm, "Chuyển mã…", tag::OPEN_CONVERT);
        let quit_btn = make_quiet_button(mtm, "Thoát", tag::QUIT);
        place(&macro_btn, 12.0, PANEL_HEIGHT - 48.0, 80.0, 36.0);
        place(&convert_btn, 96.0, PANEL_HEIGHT - 48.0, 96.0, 36.0);
        place(
            &quit_btn,
            PANEL_WIDTH - 72.0,
            PANEL_HEIGHT - 48.0,
            60.0,
            36.0,
        );
        for btn in [&macro_btn, &convert_btn, &quit_btn] {
            unsafe {
                btn.setTarget(Some(target));
                btn.setAction(Some(sel!(uiAction:)));
            }
        }

        for v in [
            &icon as &NSView,
            &title,
            &subtitle,
            &gear,
            &vi_btn,
            &en_btn,
            &hotkey_hint,
            &header,
            &input_btns[0],
            &input_btns[1],
            &input_btns[2],
            &input_btns[3],
            &hint,
            &code_popup,
            &spell_l,
            &spell_switch,
            &smart_l,
            &smart_switch,
            &banner_box,
            &smart_banner,
            &macro_btn,
            &convert_btn,
            &quit_btn,
        ] {
            root.addSubview(v);
        }

        let controller = NSViewController::new(mtm);
        controller.setView(&root);
        popover.setContentViewController(Some(&controller));
        popover.setContentSize(objc2_foundation::NSSize::new(PANEL_WIDTH, PANEL_HEIGHT));

        Self {
            popover,
            vi_btn,
            en_btn,
            hotkey_hint,
            input_btns,
            hint,
            code_popup,
            spell_switch,
            smart_switch,
            smart_banner,
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

    pub fn refresh(&self, snap: &UiSnapshot) {
        let vi = snap.settings.vLanguage != 0;
        self.vi_btn.setBordered(!vi);
        self.en_btn.setBordered(vi);
        if vi {
            self.vi_btn.setContentTintColor(Some(&accent_color()));
        } else {
            self.en_btn.setContentTintColor(Some(&accent_color()));
        }
        let sym = snap.hotkey.format_symbols();
        self.hotkey_hint
            .setStringValue(&ns(&format!("Chuyển nhanh bằng {sym}")));
        let mode = snap.settings.vInputType;
        for (i, btn) in self.input_btns.iter().enumerate() {
            let on = mode == i as i64;
            btn.setBordered(!on);
            if on {
                btn.setContentTintColor(Some(&accent_color()));
            }
        }
        self.hint.setStringValue(&ns(input_type_hint(mode)));
        let table = snap.settings.vCodeTable.clamp(0, 4);
        self.code_popup.selectItemAtIndex(table as isize);
        set_switch(&self.spell_switch, snap.spelling);
        set_switch(&self.smart_switch, snap.settings.vUseSmartSwitchKey != 0);
        if let Some(app) = &snap.focus_app {
            let lang = if snap.settings.vLanguage != 0 {
                "Tiếng Việt"
            } else {
                "English"
            };
            self.smart_banner.setStringValue(&ns(&format!(
                "Đang gõ trong {app} · NeoKey nhớ {lang} cho ứng dụng này"
            )));
        } else {
            self.smart_banner.setStringValue(&ns(""));
        }
    }
}
