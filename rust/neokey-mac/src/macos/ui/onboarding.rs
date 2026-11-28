use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{NSApplication, NSTextField, NSWindow, NSWindowStyleMask};
use objc2_foundation::{NSPoint, NSRect, NSSize};

use super::{style::*, tag};
use crate::macos::status_menu::MenuTarget;

pub struct Onboarding {
    window: Retained<NSWindow>,
    status: Retained<NSTextField>,
}

impl Onboarding {
    pub fn new(mtm: MainThreadMarker, target: &MenuTarget) -> Self {
        let width = 514.0;
        let height = 228.0;
        let rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(width, height));
        let window = unsafe { NSWindow::new(mtm) };
        window.setStyleMask(NSWindowStyleMask::Titled | NSWindowStyleMask::Closable);
        window.setContentSize(rect.size);
        window.setTitle(&ns("Quyền Trợ năng"));
        unsafe { window.setReleasedWhenClosed(false) };
        let view = SolidView::new(mtm, rect);
        let title = section_title(mtm, "Cho phép NeoKey gõ tiếng Việt");
        place(&title, MARGIN, 22.0, width - 2.0 * MARGIN, 28.0);
        view.addSubview(&title);

        let instructions = NSTextField::wrappingLabelWithString(
            &ns("Mở Cài đặt hệ thống → Quyền riêng tư & Bảo mật → Trợ năng, rồi bật NeoKey. Ứng dụng sẽ tự mở lại sau khi được cấp quyền."),
            mtm,
        );
        place(&instructions, MARGIN, 62.0, width - 2.0 * MARGIN, 54.0);
        view.addSubview(&instructions);

        let status = label(mtm, "", 12.0, true);
        place(&status, MARGIN, 122.0, width - 2.0 * MARGIN, 24.0);
        view.addSubview(&status);
        for (title, action, x, w) in [
            ("Thoát", tag::ONBOARDING_QUIT, MARGIN, 76.0),
            ("Thử lại", tag::ONBOARDING_RETRY, 220.0, 90.0),
            ("Mở Cài đặt hệ thống", tag::ONBOARDING_OPEN, 316.0, 174.0),
        ] {
            let button = make_button(mtm, title, action);
            unsafe {
                button.setTarget(Some(target));
                button.setAction(Some(sel!(uiAction:)));
            }
            place(&button, x, 166.0, w, 34.0);
            view.addSubview(&button);
        }
        window.setContentView(Some(&view));
        window.center();
        Self { window, status }
    }

    pub fn show(&self) {
        let mtm = MainThreadMarker::new().expect("main thread");
        let app = NSApplication::sharedApplication(mtm);
        #[allow(deprecated)]
        app.activateIgnoringOtherApps(true);
        self.window.makeKeyAndOrderFront(None);
    }

    pub fn poll(&self, trusted: bool) {
        if trusted && self.window.isVisible() {
            self.window.orderOut(None);
        }
    }

    pub(super) fn snapshot_view(&self) -> Option<Retained<objc2_app_kit::NSView>> {
        self.window.contentView()
    }

    pub fn set_waiting(&self) {
        self.status
            .setStringValue(&ns("Đang chờ bạn cấp quyền trong Cài đặt hệ thống."));
    }
}
