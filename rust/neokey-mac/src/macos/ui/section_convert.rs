//! Chuyển mã tool: conversion options in one card, the text pair in another.

use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{NSButton, NSPopUpButton, NSSwitch, NSTextField, NSView};
use objc2_foundation::{NSPoint, NSRect, NSSize};

use super::convert_map::{CaseMode, CODE_TABLE_LABELS};
use super::help::add_help;
use super::style::*;
use super::tag;
use super::UiSnapshot;
use crate::macos::status_menu::MenuTarget;

const FROM_TOOLTIP: &str = "Bảng mã của văn bản gốc cần chuyển.";
const TO_TOOLTIP: &str = "Bảng mã đích sau khi chuyển.";
const SWAP_TOOLTIP: &str = "Đảo hai bảng mã: Từ thành Sang và ngược lại.";
const CASE_TOOLTIP: &str =
    "Kiểu chữ của kết quả: mặc định, viết hoa hết, viết thường, hoa đầu câu hoặc hoa mỗi từ.";
/// `remove_mark` outputs the bare base letter (đ → d) and lowercases it unless
/// a case mode is chosen — verified against `convert.rs` / ConvertTool.cpp.
const REMOVE_MARK_TOOLTIP: &str =
    "Bỏ hết dấu tiếng Việt, kể cả đ thành d; kết quả về chữ thường trừ khi chọn kiểu chữ khác. \
     Vd. Việt Nam → viet nam.";
const INPUT_TOOLTIP: &str = "Dán hoặc gõ văn bản cần chuyển vào đây.";
const OUTPUT_TOOLTIP: &str = "Văn bản sau khi chuyển (chỉ đọc); dùng nút Sao chép để lấy kết quả.";
const PASTE_TOOLTIP: &str = "Dán văn bản từ clipboard vào ô Văn bản gốc rồi chuyển ngay.";
const COPY_TOOLTIP: &str =
    "Chuyển văn bản gốc theo các tuỳ chọn hiện tại rồi chép kết quả vào clipboard.";
const RUN_TOOLTIP: &str = "Chuyển văn bản gốc sang bảng mã đã chọn và hiện ở ô Kết quả.";

pub struct ConvertSection {
    view: Retained<FlippedView>,
    from_popup: Retained<NSPopUpButton>,
    to_popup: Retained<NSPopUpButton>,
    case_popup: Retained<NSPopUpButton>,
    remove_mark: Retained<NSSwitch>,
    hotkey: Retained<NSButton>,
    clear_hotkey: Retained<NSButton>,
    alert: Retained<NSSwitch>,
    input: Retained<NSTextField>,
    output: Retained<NSTextField>,
}

impl ConvertSection {
    pub fn build(mtm: MainThreadMarker, target: &MenuTarget, w: f64, h: f64) -> Self {
        let view = FlippedView::new(mtm, NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(w, h)));
        let title = section_title(mtm, "Chuyển mã");
        place(&title, MARGIN, 18.0, w - 2.0 * MARGIN, 26.0);
        view.addSubview(&title);

        let card_w = w - 2.0 * MARGIN;

        // Conversion options.
        let options_card = card(mtm, MARGIN, 58.0, card_w, 6.0 * ROW_HEIGHT + 12.0);
        let from_rect = row_with_tooltip(
            mtm,
            &options_card,
            6.0,
            "Từ bảng mã",
            FROM_TOOLTIP,
            210.0,
            26.0,
        );
        let from_popup = NSPopUpButton::new(mtm);
        for label in CODE_TABLE_LABELS {
            from_popup.addItemWithTitle(&ns(label));
        }
        from_popup.setToolTip(Some(&ns(FROM_TOOLTIP)));
        add_control(&options_card, &from_popup, from_rect);
        row_separator(mtm, &options_card, ROW_HEIGHT + 6.0);

        // Same width as "Từ bảng mã" so the two popups line up; the swap icon
        // sits just left of it.
        let to_rect = row_with_tooltip(
            mtm,
            &options_card,
            ROW_HEIGHT + 6.0,
            "Sang bảng mã",
            TO_TOOLTIP,
            210.0,
            26.0,
        );
        let to_popup = NSPopUpButton::new(mtm);
        for label in CODE_TABLE_LABELS {
            to_popup.addItemWithTitle(&ns(label));
        }
        to_popup.setToolTip(Some(&ns(TO_TOOLTIP)));
        add_control(&options_card, &to_popup, to_rect);
        let swap = make_icon_button(mtm, "arrow.up.arrow.down", "⇅", tag::CONVERT_SWAP);
        swap.setToolTip(Some(&ns(SWAP_TOOLTIP)));
        add_control(
            &options_card,
            &swap,
            NSRect::new(
                NSPoint::new(to_rect.origin.x - 36.0, to_rect.origin.y - 1.0),
                NSSize::new(28.0, 28.0),
            ),
        );
        row_separator(mtm, &options_card, 2.0 * ROW_HEIGHT + 6.0);

        let case_rect = row_with_tooltip(
            mtm,
            &options_card,
            2.0 * ROW_HEIGHT + 6.0,
            "Chữ viết",
            CASE_TOOLTIP,
            210.0,
            26.0,
        );
        let case_popup = NSPopUpButton::new(mtm);
        for mode in CaseMode::ALL {
            case_popup.addItemWithTitle(&ns(mode.label()));
        }
        case_popup.setToolTip(Some(&ns(CASE_TOOLTIP)));
        add_control(&options_card, &case_popup, case_rect);
        row_separator(mtm, &options_card, 3.0 * ROW_HEIGHT + 6.0);

        let mark_rect = row_with_tooltip(
            mtm,
            &options_card,
            3.0 * ROW_HEIGHT + 6.0,
            "Bỏ dấu",
            REMOVE_MARK_TOOLTIP,
            switch_w(mtm),
            switch_h(mtm),
        );
        let remove_mark = make_switch(mtm, tag::CONVERT_OPTIONS, false);
        remove_mark.setToolTip(Some(&ns(REMOVE_MARK_TOOLTIP)));
        add_control(&options_card, &remove_mark, mark_rect);
        row_separator(mtm, &options_card, 4.0 * ROW_HEIGHT + 6.0);
        let hotkey_tip = "Bấm để ghi phím tắt chuyển mã nội dung clipboard. Nhấn Esc để huỷ.";
        let hotkey_rect = row_with_tooltip(
            mtm,
            &options_card,
            4.0 * ROW_HEIGHT + 6.0,
            "Phím tắt chuyển mã nhanh",
            hotkey_tip,
            150.0,
            30.0,
        );
        let hotkey = make_button(mtm, "Ghi phím…", tag::CONVERT_HOTKEY);
        add_control(&options_card, &hotkey, hotkey_rect);
        let clear_hotkey = make_icon_button(mtm, "xmark", "×", tag::CONVERT_CLEAR_HOTKEY);
        clear_hotkey.setToolTip(Some(&ns("Xoá phím tắt chuyển mã nhanh")));
        place(
            &clear_hotkey,
            hotkey_rect.origin.x - 34.0,
            hotkey_rect.origin.y,
            28.0,
            30.0,
        );
        options_card.addSubview(&clear_hotkey);
        row_separator(mtm, &options_card, 5.0 * ROW_HEIGHT + 6.0);
        let alert_tip = "Hiện thông báo khi đã chuyển mã nội dung clipboard.";
        let alert_rect = row_with_tooltip(
            mtm,
            &options_card,
            5.0 * ROW_HEIGHT + 6.0,
            "Báo khi chuyển mã xong",
            alert_tip,
            switch_w(mtm),
            switch_h(mtm),
        );
        let alert = make_switch(mtm, tag::CONVERT_ALERT, false);
        alert.setToolTip(Some(&ns(alert_tip)));
        add_control(&options_card, &alert, alert_rect);
        view.addSubview(&options_card);

        // Text pair: two multi-line boxes, label above each.
        const BOX_H: f64 = 67.0;
        let text_top = 58.0 + 6.0 * ROW_HEIGHT + 12.0 + 16.0;
        let box_w = card_w - 2.0 * CARD_PAD;
        let text_card = card(
            mtm,
            MARGIN,
            text_top,
            card_w,
            2.0 * (24.0 + BOX_H + 8.0) + ROW_HEIGHT + 12.0,
        );
        let input_label = label(mtm, "Văn bản gốc", 13.0, true);
        input_label.setToolTip(Some(&ns(INPUT_TOOLTIP)));
        place(&input_label, CARD_PAD, 12.0, box_w, 18.0);
        text_card.addSubview(&input_label);
        add_help(mtm, &text_card, &input_label, INPUT_TOOLTIP);
        let input = multiline_field(mtm, true);
        input.setPlaceholderString(Some(&ns("Dán hoặc gõ văn bản cần chuyển")));
        input.setToolTip(Some(&ns(INPUT_TOOLTIP)));
        add_control(
            &text_card,
            &input,
            NSRect::new(NSPoint::new(CARD_PAD, 34.0), NSSize::new(box_w, BOX_H)),
        );
        let output_top = 34.0 + BOX_H + 8.0;
        let output_label = label(mtm, "Kết quả", 13.0, true);
        output_label.setToolTip(Some(&ns(OUTPUT_TOOLTIP)));
        place(&output_label, CARD_PAD, output_top + 4.0, box_w, 18.0);
        text_card.addSubview(&output_label);
        add_help(mtm, &text_card, &output_label, OUTPUT_TOOLTIP);
        let output = multiline_field(mtm, false);
        output.setToolTip(Some(&ns(OUTPUT_TOOLTIP)));
        add_control(
            &text_card,
            &output,
            NSRect::new(
                NSPoint::new(CARD_PAD, output_top + 26.0),
                NSSize::new(box_w, BOX_H),
            ),
        );
        let buttons_top = output_top + 26.0 + BOX_H + 8.0;
        row_separator(mtm, &text_card, buttons_top);

        let buttons_y = buttons_top + 8.0;
        let paste = make_button(mtm, "Dán", tag::CONVERT_PASTE);
        paste.setToolTip(Some(&ns(PASTE_TOOLTIP)));
        add_control(
            &text_card,
            &paste,
            NSRect::new(NSPoint::new(CARD_PAD, buttons_y), NSSize::new(70.0, 28.0)),
        );
        let copy = make_button(mtm, "Sao chép", tag::CONVERT_COPY);
        copy.setToolTip(Some(&ns(COPY_TOOLTIP)));
        add_control(
            &text_card,
            &copy,
            NSRect::new(
                NSPoint::new(CARD_PAD + 78.0, buttons_y),
                NSSize::new(110.0, 28.0),
            ),
        );
        let quick = make_button(mtm, "Chuyển mã nhanh", tag::CONVERT_QUICK);
        quick.setToolTip(Some(&ns(
            "Chuyển mã trực tiếp văn bản và HTML trong clipboard.",
        )));
        place(&quick, CARD_PAD + 193.0, buttons_y, 148.0, 28.0);
        text_card.addSubview(&quick);
        let run = make_button(mtm, "Chuyển", tag::CONVERT_RUN);
        run.setToolTip(Some(&ns(RUN_TOOLTIP)));
        add_control(
            &text_card,
            &run,
            NSRect::new(
                NSPoint::new(card_w - CARD_PAD - 90.0, buttons_y),
                NSSize::new(90.0, 28.0),
            ),
        );
        view.addSubview(&text_card);

        for control in [
            &hotkey as &objc2_app_kit::NSControl,
            &clear_hotkey,
            &alert,
            &quick,
        ] {
            unsafe {
                control.setTarget(Some(target));
                control.setAction(Some(sel!(uiAction:)));
            }
        }
        from_popup.setTag(tag::CONVERT_OPTIONS);
        to_popup.setTag(tag::CONVERT_OPTIONS);
        case_popup.setTag(tag::CONVERT_OPTIONS);
        remove_mark.setTag(tag::CONVERT_OPTIONS);
        unsafe {
            from_popup.setTarget(Some(target));
            from_popup.setAction(Some(sel!(uiAction:)));
            to_popup.setTarget(Some(target));
            to_popup.setAction(Some(sel!(uiAction:)));
            case_popup.setTarget(Some(target));
            case_popup.setAction(Some(sel!(uiAction:)));
            remove_mark.setTarget(Some(target));
            remove_mark.setAction(Some(sel!(uiAction:)));
            swap.setTarget(Some(target));
            swap.setAction(Some(sel!(uiAction:)));
            paste.setTarget(Some(target));
            paste.setAction(Some(sel!(uiAction:)));
            copy.setTarget(Some(target));
            copy.setAction(Some(sel!(uiAction:)));
            run.setTarget(Some(target));
            run.setAction(Some(sel!(uiAction:)));
        }

        Self {
            view,
            from_popup,
            to_popup,
            case_popup,
            remove_mark,
            hotkey,
            clear_hotkey,
            alert,
            input,
            output,
        }
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }

    pub fn input_text(&self) -> String {
        self.input.stringValue().to_string()
    }

    pub fn set_output(&self, text: &str) {
        self.output.setStringValue(&ns(text));
    }

    pub fn ui_options(&self) -> super::convert_map::ConvertUiOptions {
        super::convert_map::ConvertUiOptions {
            from_code: self.from_popup.indexOfSelectedItem() as u8,
            to_code: self.to_popup.indexOfSelectedItem() as u8,
            case_mode: CaseMode::ALL[self.case_popup.indexOfSelectedItem().clamp(0, 4) as usize],
            remove_mark: switch_is_on(&self.remove_mark),
        }
    }

    pub fn set_input(&self, text: &str) {
        self.input.setStringValue(&ns(text));
    }

    pub fn set_from_to(&self, from: u8, to: u8) {
        self.from_popup.selectItemAtIndex(from as isize);
        self.to_popup.selectItemAtIndex(to as isize);
    }

    pub fn set_options(&self, options: super::convert_map::ConvertUiOptions) {
        self.set_from_to(options.from_code, options.to_code);
        self.case_popup
            .selectItemAtIndex(options.case_mode as isize);
        set_switch(&self.remove_mark, options.remove_mark);
    }

    pub fn set_capturing(&self) {
        self.hotkey.setTitle(&ns("Nhấn tổ hợp…"));
    }

    pub fn refresh(&self, snap: &UiSnapshot) {
        let chord = snap.convert_hotkey.format_symbols();
        self.hotkey.setTitle(&ns(if chord == "—" {
            "Ghi phím…"
        } else {
            &chord
        }));
        self.hotkey
            .setToolTip(Some(&ns(&snap.convert_hotkey.format_labels())));
        self.hotkey.setEnabled(snap.tap_ready);
        self.clear_hotkey.setEnabled(chord != "—");
        set_switch(&self.alert, snap.convert_alert);
    }
}

/// A wrapping, multi-line text box (NSTextField keeps the existing
/// `stringValue` plumbing; Option-Return inserts a newline while typing).
fn multiline_field(mtm: MainThreadMarker, editable: bool) -> Retained<NSTextField> {
    let field = NSTextField::new(mtm);
    field.setEditable(editable);
    field.setSelectable(true);
    field.setUsesSingleLineMode(false);
    field.setLineBreakMode(objc2_app_kit::NSLineBreakMode::ByWordWrapping);
    if let Some(cell) = field.cell() {
        cell.setWraps(true);
        cell.setScrollable(false);
    }
    field
}
