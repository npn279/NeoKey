//! Kiểu gõ preferences: input type, code table and grouped option cards.

use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{
    NSPopUpButton, NSScrollView, NSSegmentDistribution, NSSegmentSwitchTracking,
    NSSegmentedControl, NSSwitch, NSTextField, NSView,
};
use objc2_foundation::{NSPoint, NSRect, NSSize};

use super::convert_map::{input_type_hint, CODE_TABLE_LABELS, INPUT_TYPE_LABELS};
use super::style::*;
use super::tag;
use super::UiSnapshot;
use crate::macos::status_menu::MenuTarget;

/// Preference groups, each rendered as one card. The offsets keep the
/// `tag::SETTING + offset` plumbing in `macos::handle_ui_action` unchanged.
const GROUPS: &[(&str, &[(&str, isize)])] = &[
    (
        "Chính tả",
        &[
            ("Kiểm tra chính tả", 10),
            ("Chính tả hiện đại", 11),
            ("Khôi phục khi sai chính tả", 13),
        ],
    ),
    (
        "Gõ nhanh",
        &[
            ("Telex nhanh", 12),
            ("Gõ nhanh phụ âm đầu (f→ph, j→gi, w→qu)", 16),
            ("Gõ nhanh phụ âm cuối (g→ng, h→nh, k→ch)", 17),
            ("Cho phép phụ âm z + ZWJ", 15),
        ],
    ),
    (
        "Gõ tắt",
        &[
            ("Dùng macro", 18),
            ("Macro trong chế độ Anh", 19),
            ("Tự viết hoa macro", 20),
        ],
    ),
    (
        "Khác",
        &[
            ("Viết hoa chữ cái đầu", 14),
            ("Chặn gõ khi bàn phím không phải tiếng Anh", 21),
            ("Tắt tạm NeoKey", 22),
        ],
    ),
];

pub struct TypingSection {
    view: Retained<FlippedView>,
    input_segment: Retained<NSSegmentedControl>,
    input_hint: Retained<NSTextField>,
    code_popup: Retained<NSPopUpButton>,
    switches: Vec<(isize, Retained<NSSwitch>)>,
}

impl TypingSection {
    pub fn build(mtm: MainThreadMarker, target: &MenuTarget, w: f64, h: f64) -> Self {
        let scroll = NSScrollView::new(mtm);
        place(&scroll, 0.0, 0.0, w, h);
        scroll.setDrawsBackground(false);
        let doc = FlippedView::new(
            mtm,
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(w, 1.0)),
        );
        let title = section_title(mtm, "Kiểu gõ");
        place(&title, MARGIN, 18.0, w - 2.0 * MARGIN, 26.0);
        doc.addSubview(&title);

        let card_w = w - 2.0 * MARGIN;
        let mut stack = Stack::new(58.0);

        // Input type and code table (moved here from the control panel).
        let input_card = card(mtm, MARGIN, stack.take(126.0), card_w, 126.0);
        let style_label = label(mtm, "Kiểu gõ", 14.0, false);
        place(&style_label, CARD_PAD, 18.0, 120.0, 20.0);
        input_card.addSubview(&style_label);
        let input_segment = NSSegmentedControl::new(mtm);
        input_segment.setSegmentCount(4);
        for (i, name) in INPUT_TYPE_LABELS.iter().enumerate() {
            input_segment.setLabel_forSegment(&ns(name), i as isize);
        }
        input_segment.setTrackingMode(NSSegmentSwitchTracking::SelectOne);
        input_segment.setSegmentDistribution(NSSegmentDistribution::FillProportionally);
        input_segment.setTag(tag::INPUT_TYPE);
        add_control(
            &input_card,
            &input_segment,
            NSRect::new(
                NSPoint::new(card_w - CARD_PAD - 340.0, 12.0),
                NSSize::new(340.0, 32.0),
            ),
        );
        unsafe {
            input_segment.setTarget(Some(target));
            input_segment.setAction(Some(sel!(uiAction:)));
        }
        // Full-width hint line under the input row (the hint is too long to sit
        // beside the label without truncating).
        let input_hint = label(mtm, "", 11.0, true);
        place(&input_hint, CARD_PAD, 52.0, card_w - 2.0 * CARD_PAD, 16.0);
        input_card.addSubview(&input_hint);
        row_separator(mtm, &input_card, 72.0);
        let code_rect = row(mtm, &input_card, 76.0, "Bảng mã", 210.0, 26.0);
        let code_popup = NSPopUpButton::new(mtm);
        for name in CODE_TABLE_LABELS {
            code_popup.addItemWithTitle(&ns(name));
        }
        code_popup.setTag(tag::CODE_TABLE);
        add_control(&input_card, &code_popup, code_rect);
        unsafe {
            code_popup.setTarget(Some(target));
            code_popup.setAction(Some(sel!(uiAction:)));
        }
        doc.addSubview(&input_card);

        // Grouped preference cards.
        let mut switches = Vec::new();
        for (group, rows) in GROUPS {
            stack.gap(18.0);
            let header = section_header(mtm, group);
            place(&header, MARGIN, stack.take(16.0), card_w, 16.0);
            doc.addSubview(&header);
            stack.gap(6.0);
            let card_h = rows.len() as f64 * ROW_HEIGHT + 12.0;
            let group_card = card(mtm, MARGIN, stack.take(card_h), card_w, card_h);
            for (i, (text, offset)) in rows.iter().enumerate() {
                let row_y = 6.0 + i as f64 * ROW_HEIGHT;
                if i > 0 {
                    row_separator(mtm, &group_card, row_y);
                }
                let rect = row(mtm, &group_card, row_y, text, switch_w(mtm), switch_h(mtm));
                let switch = make_switch(mtm, tag::SETTING + offset, false);
                add_control(&group_card, &switch, rect);
                unsafe {
                    switch.setTarget(Some(target));
                    switch.setAction(Some(sel!(uiAction:)));
                }
                switches.push((*offset, switch));
            }
            doc.addSubview(&group_card);
        }
        stack.gap(MARGIN);
        doc.setFrameSize(NSSize::new(w, stack.bottom()));
        scroll.setDocumentView(Some(&doc));

        let view = FlippedView::new(mtm, NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(w, h)));
        view.addSubview(&scroll);
        Self {
            view,
            input_segment,
            input_hint,
            code_popup,
            switches,
        }
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }

    pub fn refresh(&self, snap: &UiSnapshot, dark: bool) {
        let accent = accent_color(dark);
        let mode = snap.settings.vInputType.clamp(0, 3);
        self.input_segment.setSelectedSegment(mode as isize);
        self.input_segment
            .setSelectedSegmentBezelColor(Some(&accent));
        self.input_hint.setStringValue(&ns(input_type_hint(mode)));
        self.code_popup
            .selectItemAtIndex(snap.settings.vCodeTable.clamp(0, 4) as isize);
        for (offset, switch) in &self.switches {
            set_switch(switch, pref_value(snap, *offset));
        }
    }
}

fn pref_value(snap: &UiSnapshot, offset: isize) -> bool {
    let settings = &snap.settings;
    match offset {
        10 => snap.spelling,
        11 => settings.vUseModernOrthography != 0,
        12 => settings.vQuickTelex != 0,
        13 => settings.vRestoreIfWrongSpelling != 0,
        14 => settings.vUpperCaseFirstChar != 0,
        15 => settings.vAllowConsonantZFWJ != 0,
        16 => settings.vQuickStartConsonant != 0,
        17 => settings.vQuickEndConsonant != 0,
        18 => settings.vUseMacro != 0,
        19 => settings.vUseMacroInEnglishMode != 0,
        20 => settings.vAutoCapsMacro != 0,
        21 => settings.vOtherLanguage != 0,
        22 => settings.vTempOffOpenKey != 0,
        _ => false,
    }
}
