//! Gõ tắt (macros): add form in a card, the macro list in a scrolling card.

use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{NSColor, NSScrollView, NSScrollerStyle, NSTextField, NSView};
use objc2_foundation::{NSPoint, NSRect, NSSize};

use super::style::*;
use super::tag;
use super::UiSnapshot;
use crate::macos::status_menu::MenuTarget;

const SHORTCUT_TOOLTIP: &str =
    "Chữ tắt sẽ gõ, vd. vn. Gõ chữ này rồi nhấn dấu cách để thay bằng cụm đầy đủ.";
const CONTENT_TOOLTIP: &str = "Cụm từ đầy đủ sẽ thay cho chữ tắt, vd. Việt Nam.";
const ADD_TOOLTIP: &str = "Thêm gõ tắt mới từ hai ô Chữ tắt và Thay bằng.";
const IMPORT_TOOLTIP: &str =
    "Nhập danh sách gõ tắt từ tệp (định dạng UniKey/OpenKey), thay cho danh sách hiện tại.";
const EXPORT_TOOLTIP: &str =
    "Lưu danh sách gõ tắt ra tệp (định dạng UniKey/OpenKey) để sao lưu hoặc chuyển sang máy khác.";

pub struct MacrosSection {
    view: Retained<FlippedView>,
    shortcut: Retained<NSTextField>,
    content: Retained<NSTextField>,
    list: Retained<FlippedView>,
    empty: Retained<NSTextField>,
}

impl MacrosSection {
    pub fn build(mtm: MainThreadMarker, target: &MenuTarget, w: f64, h: f64) -> Self {
        let view = FlippedView::new(mtm, NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(w, h)));
        let title = section_title(mtm, "Gõ tắt");
        place(&title, MARGIN, 18.0, w - 2.0 * MARGIN, 26.0);
        view.addSubview(&title);

        let hint = label(
            mtm,
            "Gõ chữ tắt rồi nhấn dấu cách để thay bằng cụm đầy đủ.",
            12.0,
            true,
        );
        place(&hint, MARGIN, 50.0, w - 2.0 * MARGIN, 16.0);
        view.addSubview(&hint);

        let card_w = w - 2.0 * MARGIN;
        let card_h = 3.0 * ROW_HEIGHT + 12.0;
        let form_card = card(mtm, MARGIN, 74.0, card_w, card_h);

        // Both fields start at the same x; the shortcut field is just shorter.
        let field_x = CARD_PAD + 110.0;
        let field_w = card_w - field_x - CARD_PAD;
        let mut shortcut_rect = row_with_tooltip(
            mtm,
            &form_card,
            6.0,
            "Chữ tắt",
            SHORTCUT_TOOLTIP,
            field_w,
            28.0,
        );
        shortcut_rect.size.width = 180.0;
        let shortcut = NSTextField::new(mtm);
        shortcut.setPlaceholderString(Some(&ns("vd: vn")));
        shortcut.setToolTip(Some(&ns(SHORTCUT_TOOLTIP)));
        add_control(&form_card, &shortcut, shortcut_rect);
        row_separator(mtm, &form_card, ROW_HEIGHT + 6.0);
        let content_rect = row_with_tooltip(
            mtm,
            &form_card,
            ROW_HEIGHT + 6.0,
            "Thay bằng",
            CONTENT_TOOLTIP,
            field_w,
            28.0,
        );
        let content = NSTextField::new(mtm);
        content.setPlaceholderString(Some(&ns("vd: Việt Nam")));
        content.setToolTip(Some(&ns(CONTENT_TOOLTIP)));
        add_control(&form_card, &content, content_rect);
        row_separator(mtm, &form_card, 2.0 * ROW_HEIGHT + 6.0);

        // Action row: import/export on the left, the primary "Thêm" on the right.
        // Deleting is per row in the list below.
        let buttons_y = 2.0 * ROW_HEIGHT + 6.0 + 8.0;
        let import_btn = make_button(mtm, "Nhập…", tag::MACRO_IMPORT);
        import_btn.setToolTip(Some(&ns(IMPORT_TOOLTIP)));
        add_control(
            &form_card,
            &import_btn,
            NSRect::new(NSPoint::new(CARD_PAD, buttons_y), NSSize::new(84.0, 28.0)),
        );
        let export_btn = make_button(mtm, "Xuất…", tag::MACRO_EXPORT);
        export_btn.setToolTip(Some(&ns(EXPORT_TOOLTIP)));
        add_control(
            &form_card,
            &export_btn,
            NSRect::new(
                NSPoint::new(CARD_PAD + 92.0, buttons_y),
                NSSize::new(84.0, 28.0),
            ),
        );
        let add = make_button(mtm, "Thêm", tag::MACRO_ADD);
        add.setToolTip(Some(&ns(ADD_TOOLTIP)));
        add_control(
            &form_card,
            &add,
            NSRect::new(
                NSPoint::new(card_w - CARD_PAD - 90.0, buttons_y),
                NSSize::new(90.0, 28.0),
            ),
        );
        view.addSubview(&form_card);

        // Macro list in a scrolling card.
        let list_top = 74.0 + card_h + 16.0;
        let list_h = h - list_top - MARGIN;
        let list_card = card(mtm, MARGIN, list_top, card_w, list_h);
        let scroll = NSScrollView::new(mtm);
        place(&scroll, 0.0, 0.0, card_w, list_h);
        scroll.setHasVerticalScroller(true);
        scroll.setDrawsBackground(false);
        scroll.setScrollerStyle(NSScrollerStyle::Overlay);
        let list = FlippedView::new(
            mtm,
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(card_w, 1.0)),
        );
        scroll.setDocumentView(Some(&list));
        list_card.addSubview(&scroll);
        let empty = label(
            mtm,
            "Chưa có gõ tắt nào. Thêm một chữ tắt ở trên.",
            13.0,
            true,
        );
        empty.setAlignment(objc2_app_kit::NSTextAlignment::Center);
        place(
            &empty,
            CARD_PAD,
            (list_h - 20.0) / 2.0,
            card_w - 2.0 * CARD_PAD,
            20.0,
        );
        list_card.addSubview(&empty);
        view.addSubview(&list_card);

        unsafe {
            import_btn.setTarget(Some(target));
            import_btn.setAction(Some(sel!(uiAction:)));
            export_btn.setTarget(Some(target));
            export_btn.setAction(Some(sel!(uiAction:)));
            add.setTarget(Some(target));
            add.setAction(Some(sel!(uiAction:)));
        }

        Self {
            view,
            shortcut,
            content,
            list,
            empty,
        }
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }

    pub fn shortcut_text(&self) -> String {
        self.shortcut.stringValue().to_string()
    }

    pub fn content_text(&self) -> String {
        self.content.stringValue().to_string()
    }

    pub fn clear_fields(&self) {
        self.shortcut.setStringValue(&ns(""));
        self.content.setStringValue(&ns(""));
    }

    pub fn refresh(&self, snap: &UiSnapshot, target: &MenuTarget) {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        // subviews() returns a snapshot copy, so remove each view it lists.
        for view in self.list.subviews() {
            view.removeFromSuperview();
        }
        self.empty.setHidden(!snap.macros.is_empty());
        let width = self.list.bounds().size.width.max(1.0);
        for (i, (key, value)) in snap.macros.iter().enumerate() {
            let y = i as f64 * ROW_HEIGHT;
            // Hovering either label explains the whole pair.
            let pair_tooltip = ns(&format!("Gõ \"{key}\" rồi dấu cách → \"{value}\"."));
            let key_lab = label(mtm, key, 13.0, false);
            key_lab.setToolTip(Some(&pair_tooltip));
            place(&key_lab, CARD_PAD, y + 12.0, 140.0, 20.0);
            let val_lab = label(mtm, value, 13.0, false);
            val_lab.setToolTip(Some(&pair_tooltip));
            place(&val_lab, 164.0, y + 12.0, width - 254.0, 20.0);
            let del = make_quiet_button(mtm, "Xóa", tag::MACRO_ROW_DELETE + i as isize);
            del.setToolTip(Some(&ns(&format!("Xóa gõ tắt \"{key}\"."))));
            place(&del, width - CARD_PAD - 56.0, y + 8.0, 56.0, 28.0);
            del.setContentTintColor(Some(&NSColor::secondaryLabelColor()));
            unsafe {
                del.setTarget(Some(target));
                del.setAction(Some(sel!(uiAction:)));
            }
            self.list.addSubview(&key_lab);
            self.list.addSubview(&val_lab);
            self.list.addSubview(&del);
            if i + 1 < snap.macros.len() {
                let separator = separator_box(
                    mtm,
                    NSRect::new(
                        NSPoint::new(CARD_PAD, y + ROW_HEIGHT - 0.5),
                        NSSize::new(width - 2.0 * CARD_PAD, 1.0),
                    ),
                );
                self.list.addSubview(&separator);
            }
        }
        self.list.setFrameSize(NSSize::new(
            width,
            snap.macros.len().max(1) as f64 * ROW_HEIGHT,
        ));
    }
}
