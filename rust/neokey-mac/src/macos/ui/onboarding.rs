use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{NSApplication, NSButton, NSView, NSWindow, NSWindowStyleMask};

use super::style::*;
use super::tag;
use crate::macos::status_menu::MenuTarget;

pub struct Onboarding {
    window: Retained<NSWindow>,
    status: Retained<objc2_app_kit::NSTextField>,
}

impl Onboarding {
    pub fn new(mtm: MainThreadMarker, target: &MenuTarget) -> Self {
        let dark = is_dark(&NSApplication::sharedApplication(mtm).effectiveAppearance());
        let rect = objc2_foundation::NSRect::new(
            objc2_foundation::NSPoint::new(0.0, 0.0),
            objc2_foundation::NSSize::new(ONBOARDING_WIDTH, ONBOARDING_HEIGHT),
        );
        let style = NSWindowStyleMask::Titled | NSWindowStyleMask::Closable;
        let window = unsafe { NSWindow::new(mtm) };
        window.setFrame_display(rect, true);
        window.setStyleMask(style);
        window.setTitle(&ns("NeoKey"));
        unsafe {
            window.setReleasedWhenClosed(false);
        }
        window.center();

        let root = FlippedView::new(mtm, rect);
        let title = title_label(mtm, "Còn một bước nữa là gõ được tiếng Việt", 24.0);
        place(&title, 56.0, 44.0, ONBOARDING_WIDTH - 112.0, 32.0);
        let body = label(
            mtm,
            "NeoKey cần quyền Trợ năng để đọc phím bạn gõ và thêm dấu.",
            14.0,
            true,
        );
        place(&body, 56.0, 80.0, ONBOARDING_WIDTH - 112.0, 20.0);

        let open = make_accent_button(mtm, "Mở Cài đặt", tag::ONBOARDING_OPEN, dark);
        place(&open, ONBOARDING_WIDTH - 220.0, 160.0, 140.0, 38.0);
        let retry = make_button(mtm, "Kiểm tra lại", tag::ONBOARDING_RETRY);
        let quit = make_button(mtm, "Thoát", tag::ONBOARDING_QUIT);
        place(
            &retry,
            ONBOARDING_WIDTH - 200.0,
            ONBOARDING_HEIGHT - 60.0,
            110.0,
            38.0,
        );
        place(
            &quit,
            ONBOARDING_WIDTH - 80.0,
            ONBOARDING_HEIGHT - 60.0,
            60.0,
            38.0,
        );
        let status = label(mtm, "Đang chờ quyền Trợ năng…", 13.0, true);
        place(&status, 56.0, ONBOARDING_HEIGHT - 56.0, 300.0, 20.0);

        for btn in [&open, &retry, &quit] {
            unsafe {
                btn.setTarget(Some(target));
                btn.setAction(Some(sel!(uiAction:)));
            }
        }
        for v in [&title as &NSView, &body, &status] {
            root.addSubview(v);
        }
        for v in [&open as &NSView, &retry, &quit] {
            root.addSubview(v);
        }
        window.setContentView(Some(&root));
        Self { window, status }
    }

    pub fn show(&self) {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };
        let app = NSApplication::sharedApplication(mtm);
        app.activate();
        self.window.makeKeyAndOrderFront(None);
    }

    pub fn poll(&self, trusted: bool) {
        if trusted {
            self.status
                .setStringValue(&ns("Đã có quyền — đang kích hoạt…"));
            self.window.close();
        }
    }

    pub fn set_waiting(&self) {
        self.status.setStringValue(&ns("Đang chờ quyền Trợ năng…"));
    }
}

fn make_accent_button(
    mtm: MainThreadMarker,
    title: &str,
    tag: isize,
    dark: bool,
) -> Retained<NSButton> {
    let button = make_button(mtm, title, tag);
    button.setContentTintColor(Some(&accent_color(dark)));
    button
}
