use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{NSBorderType, NSScrollView, NSTextField, NSView};

use super::style::*;
use super::tag;
use super::UiSnapshot;
use crate::macos::status_menu::MenuTarget;

pub struct MacrosSection {
    view: Retained<FlippedView>,
    shortcut: Retained<NSTextField>,
    content: Retained<NSTextField>,
    list: Retained<FlippedView>,
    scroll: Retained<NSScrollView>,
}

impl MacrosSection {
    pub fn build(mtm: MainThreadMarker, target: &MenuTarget, w: f64, h: f64) -> Self {
        let view = FlippedView::new(
            mtm,
            objc2_foundation::NSRect::new(
                objc2_foundation::NSPoint::new(0.0, 0.0),
                objc2_foundation::NSSize::new(w, h),
            ),
        );
        let title = title_label(mtm, "Gõ tắt", 22.0);
        place(&title, 24.0, 16.0, 200.0, 30.0);
        view.addSubview(&title);

        let hint = label(
            mtm,
            "Gõ chữ tắt rồi nhấn dấu cách, NeoKey sẽ thay bằng cụm đầy đủ.",
            13.0,
            true,
        );
        place(&hint, 24.0, 48.0, w - 48.0, 18.0);
        view.addSubview(&hint);

        let shortcut = NSTextField::new(mtm);
        shortcut.setPlaceholderString(Some(&ns("vd: vn")));
        place(&shortcut, 24.0, 80.0, 140.0, 28.0);
        let content = NSTextField::new(mtm);
        content.setPlaceholderString(Some(&ns("vd: Việt Nam")));
        place(&content, 176.0, 80.0, w - 420.0, 28.0);

        let add = make_button(mtm, "Thêm", tag::MACRO_ADD);
        let del = make_button(mtm, "Xóa", tag::MACRO_DELETE);
        let import_btn = make_button(mtm, "Nhập…", tag::MACRO_IMPORT);
        let export_btn = make_button(mtm, "Xuất…", tag::MACRO_EXPORT);
        place(&add, w - 280.0, 78.0, 60.0, 28.0);
        place(&del, w - 210.0, 78.0, 60.0, 28.0);
        place(&import_btn, w - 140.0, 78.0, 60.0, 28.0);
        place(&export_btn, w - 70.0, 78.0, 60.0, 28.0);
        for btn in [&add, &del, &import_btn, &export_btn] {
            unsafe {
                btn.setTarget(Some(target));
                btn.setAction(Some(sel!(uiAction:)));
            }
            view.addSubview(btn);
        }

        let scroll = NSScrollView::new(mtm);
        place(&scroll, 24.0, 120.0, w - 48.0, h - 140.0);
        scroll.setHasVerticalScroller(true);
        scroll.setBorderType(NSBorderType::LineBorder);
        let list = FlippedView::new(
            mtm,
            objc2_foundation::NSRect::new(
                objc2_foundation::NSPoint::new(0.0, 0.0),
                objc2_foundation::NSSize::new(w - 48.0, 40.0),
            ),
        );
        scroll.setDocumentView(Some(&list));

        view.addSubview(&shortcut);
        view.addSubview(&content);
        view.addSubview(&scroll);

        Self {
            view,
            shortcut,
            content,
            list,
            scroll,
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
        let width = self.scroll.bounds().size.width.max(1.0);
        for (i, (key, value)) in snap.macros.iter().enumerate() {
            let y = i as f64 * 36.0;
            let key_lab = label(mtm, key, 13.0, false);
            place(&key_lab, 12.0, y + 8.0, 120.0, 20.0);
            let val_lab = label(mtm, value, 13.0, false);
            place(&val_lab, 140.0, y + 8.0, width - 220.0, 20.0);
            let del = make_quiet_button(mtm, "Xóa", tag::MACRO_ROW_DELETE + i as isize);
            place(&del, width - 64.0, y + 4.0, 52.0, 28.0);
            unsafe {
                del.setTarget(Some(target));
                del.setAction(Some(sel!(uiAction:)));
            }
            self.list.addSubview(&key_lab);
            self.list.addSubview(&val_lab);
            self.list.addSubview(&del);
        }
        self.list.setFrameSize(objc2_foundation::NSSize::new(
            width,
            (snap.macros.len().max(1) as f64) * 36.0,
        ));
    }
}
