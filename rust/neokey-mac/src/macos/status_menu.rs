use objc2::rc::Retained;
use objc2::runtime::{NSObject, ProtocolObject};
use objc2::{define_class, msg_send, sel, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationDelegate, NSControl, NSEventMask, NSEventType, NSImage, NSMenu,
    NSMenuItem, NSRunningApplication, NSStatusBar, NSStatusBarButton, NSStatusItem,
    NSVariableStatusItemLength, NSWorkspaceApplicationKey,
};
use objc2_foundation::{ns_string, NSNotification, NSObjectProtocol, NSTimer};

use super::{with_state, State};

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    pub(super) struct MenuTarget;

    unsafe impl NSObjectProtocol for MenuTarget {}

    unsafe impl NSApplicationDelegate for MenuTarget {
        #[unsafe(method(applicationShouldHandleReopen:hasVisibleWindows:))]
        fn reopen(&self, _: &NSApplication, _: bool) -> bool {
            with_state(|state| state.show_settings(super::ui::SettingsSection::General));
            true
        }

        #[unsafe(method(applicationWillTerminate:))]
        fn will_terminate(&self, _: &NSNotification) {
            with_state(State::shutdown);
        }
    }

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
            with_state(|state| state.handle_ui_action(tag, sender));
        }

        #[unsafe(method(quickConvert:))]
        fn quick_convert(&self, _: &NSMenuItem) {
            with_state(State::schedule_quick_convert);
        }

        #[unsafe(method(showNeoKeySettings:))]
        fn settings(&self, _: &NSMenuItem) {
            with_state(|state| state.show_settings(super::ui::SettingsSection::General));
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
    pub fn set_app_delegate(&self, app: &NSApplication) {
        app.setDelegate(Some(ProtocolObject::from_ref(self)));
    }

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
    icons: [[Option<Retained<NSImage>>; 2]; 2],
    quick_item: Retained<NSMenuItem>,
    appearance: std::cell::Cell<Option<(bool, bool)>>,
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
        let icons = [
            [
                NSImage::imageNamed(ns_string!("StatusEng")),
                NSImage::imageNamed(ns_string!("StatusHighlightedEng")),
            ],
            [
                NSImage::imageNamed(ns_string!("Status")),
                NSImage::imageNamed(ns_string!("StatusHighlighted")),
            ],
        ];
        let quick_item = install_application_menu(mtm, target);
        Ok(Self {
            status,
            button,
            icons,
            quick_item,
            appearance: std::cell::Cell::new(None),
        })
    }

    pub(super) fn remove(&self) {
        NSStatusBar::systemStatusBar().removeStatusItem(&self.status);
    }

    pub fn refresh_quick_item(&self, title: &objc2_foundation::NSString) {
        self.quick_item.setTitle(title);
    }

    pub fn refresh_title(&self, vietnamese: bool, gray: bool) {
        if self.appearance.replace(Some((vietnamese, gray))) == Some((vietnamese, gray)) {
            return;
        }
        let [image, highlighted] = &self.icons[usize::from(vietnamese)];
        if let Some(image) = image {
            image.setTemplate(gray);
            self.button.setImage(Some(image));
            self.button.setAlternateImage(highlighted.as_deref());
            self.button.setTitle(ns_string!(""));
            return;
        }
        self.button.setTitle(if vietnamese {
            ns_string!("V")
        } else {
            ns_string!("E")
        });
    }
}

fn install_application_menu(mtm: MainThreadMarker, target: &MenuTarget) -> Retained<NSMenuItem> {
    let root = NSMenu::new(mtm);
    let app_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), ns_string!("NeoKey"));
    let edit_menu = NSMenu::initWithTitle(NSMenu::alloc(mtm), ns_string!("Sửa"));
    // Standard Edit actions are resolved by the responder chain for text fields.
    unsafe {
        let app =
            root.addItemWithTitle_action_keyEquivalent(ns_string!("NeoKey"), None, ns_string!(""));
        app.setSubmenu(Some(&app_menu));
        let edit =
            root.addItemWithTitle_action_keyEquivalent(ns_string!("Sửa"), None, ns_string!(""));
        edit.setSubmenu(Some(&edit_menu));
        for (title, selector, key) in [
            (ns_string!("Cắt"), sel!(cut:), ns_string!("x")),
            (ns_string!("Sao chép"), sel!(copy:), ns_string!("c")),
            (ns_string!("Dán"), sel!(paste:), ns_string!("v")),
            (ns_string!("Chọn tất cả"), sel!(selectAll:), ns_string!("a")),
        ] {
            edit_menu.addItemWithTitle_action_keyEquivalent(title, Some(selector), key);
        }
        let settings = app_menu.addItemWithTitle_action_keyEquivalent(
            ns_string!("Cài đặt…"),
            Some(sel!(showNeoKeySettings:)),
            ns_string!(","),
        );
        settings.setTarget(Some(target));
        let quick = app_menu.addItemWithTitle_action_keyEquivalent(
            ns_string!("Chuyển mã nhanh"),
            Some(sel!(quickConvert:)),
            ns_string!(""),
        );
        quick.setTarget(Some(target));
        app_menu.addItem(&NSMenuItem::separatorItem(mtm));
        let quit = app_menu.addItemWithTitle_action_keyEquivalent(
            ns_string!("Thoát NeoKey"),
            Some(sel!(quitNeoKey:)),
            ns_string!("q"),
        );
        quit.setTarget(Some(target));
        NSApplication::sharedApplication(mtm).setMainMenu(Some(&root));
        quick
    }
}
