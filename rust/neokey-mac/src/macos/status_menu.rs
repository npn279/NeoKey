use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{define_class, msg_send, sel, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSControl, NSEventMask, NSEventType, NSMenuItem, NSRunningApplication,
    NSStatusBar, NSStatusBarButton, NSStatusItem, NSVariableStatusItemLength,
    NSWorkspaceApplicationKey,
};
use objc2_foundation::{ns_string, NSNotification, NSTimer};

use super::{with_state, State};

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    pub(super) struct MenuTarget;

    impl MenuTarget {
        #[unsafe(method(statusClicked:))]
        fn status_clicked(&self, button: &NSStatusBarButton) {
            let app = NSApplication::sharedApplication(self.mtm());
            let right = app
                .currentEvent()
                .is_some_and(|event| event.r#type() == NSEventType::RightMouseUp);
            if right {
                with_state(|state| state.show_panel(button));
            } else {
                with_state(|state| state.toggle_language(false));
            }
        }

        #[unsafe(method(uiAction:))]
        fn ui_action(&self, sender: &NSControl) {
            let tag = sender.tag();
            if tag == super::ui::tag::HOTKEY_CAPTURE {
                with_state(State::toggle_hotkey_capture);
                return;
            }
            with_state(|state| state.handle_ui_action(tag, sender));
        }

        #[unsafe(method(quitNeoKey:))]
        fn quit(&self, _: &NSMenuItem) {
            self.terminate_app();
        }

        #[unsafe(method(watchdog:))]
        fn watchdog(&self, _: &NSTimer) {
            if with_state(State::watchdog) == Some(true) {
                self.terminate_app();
            }
        }

        #[unsafe(method(appActivated:))]
        fn activated(&self, notification: &NSNotification) {
            if let Some(app) = notification_app(notification) {
                with_state(|state| state.activate_app(&app));
            }
        }

        #[unsafe(method(appTerminated:))]
        fn terminated(&self, notification: &NSNotification) {
            if let Some(app) = notification_app(notification) {
                with_state(|state| state.forget_app(app.processIdentifier()));
            }
        }
    }
);

impl MenuTarget {
    fn terminate_app(&self) {
        with_state(State::shutdown);
        NSApplication::sharedApplication(self.mtm()).terminate(None);
    }

    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![super(Self::alloc(mtm).set_ivars(())), init] }
    }
}

fn notification_app(notification: &NSNotification) -> Option<Retained<NSRunningApplication>> {
    notification
        .userInfo()?
        .objectForKey(unsafe { NSWorkspaceApplicationKey })?
        .downcast::<NSRunningApplication>()
        .ok()
}

pub struct StatusMenu {
    status: Retained<NSStatusItem>,
    button: Retained<NSStatusBarButton>,
}

impl StatusMenu {
    pub fn new(mtm: MainThreadMarker, target: &MenuTarget) -> Result<Self, &'static str> {
        let status =
            NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);
        let button = status
            .button(mtm)
            .ok_or("Cannot create NeoKey menu bar button")?;
        unsafe {
            button.setTarget(Some(target));
            button.setAction(Some(sel!(statusClicked:)));
        }
        button.sendActionOn(NSEventMask::LeftMouseUp | NSEventMask::RightMouseUp);
        button.setToolTip(Some(ns_string!(
            "NeoKey: bấm để chuyển Việt ↔ English, bấm chuột phải để mở bảng điều khiển"
        )));
        Ok(Self { status, button })
    }

    pub(super) fn remove(&self) {
        NSStatusBar::systemStatusBar().removeStatusItem(&self.status);
    }

    pub fn refresh_title(&self, vietnamese: bool) {
        self.button.setTitle(if vietnamese {
            ns_string!("V")
        } else {
            ns_string!("E")
        });
    }
}
