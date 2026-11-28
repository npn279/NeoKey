use block2::RcBlock;
use objc2::rc::Retained;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSAlert, NSAlertFirstButtonReturn, NSWindow, NSWorkspace};
use objc2_foundation::{NSRunLoop, NSURL};

use super::style::ns;

pub fn defer(action: impl Fn() + 'static) {
    // Called on the main thread. The run loop copies the block and invokes it
    // after the current AppKit action or event-tap callback has returned.
    unsafe { NSRunLoop::mainRunLoop().performBlock(&RcBlock::new(action)) };
}

pub fn open_url(url: &str) {
    if let Some(url) = NSURL::URLWithString(&ns(url)) {
        NSWorkspace::sharedWorkspace().openURL(&url);
    }
}

pub fn ask(
    parent: Option<Retained<NSWindow>>,
    title: String,
    detail: &'static str,
    accept: &'static str,
    confirm: Option<fn()>,
) {
    defer(move || {
        let alert = NSAlert::new(MainThreadMarker::new().expect("main thread"));
        alert.setMessageText(&ns(&title));
        alert.setInformativeText(&ns(detail));
        alert.addButtonWithTitle(&ns(accept));
        if confirm.is_some() {
            alert.addButtonWithTitle(&ns("Huỷ"));
        }
        let on_response = move |response| {
            if response == NSAlertFirstButtonReturn {
                if let Some(confirm) = confirm {
                    confirm();
                }
            }
        };
        if let Some(parent) = parent.as_ref().filter(|window| window.isVisible()) {
            alert.beginSheetModalForWindow_completionHandler(
                parent,
                Some(&RcBlock::new(on_response)),
            );
        } else {
            alert.window().makeKeyAndOrderFront(None);
            on_response(alert.runModal());
        }
    });
}
