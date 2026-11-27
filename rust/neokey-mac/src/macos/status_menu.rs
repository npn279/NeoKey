use neokey_engine::Settings;
use objc2::rc::Retained;
use objc2::runtime::{NSObject, Sel};
use objc2::{define_class, msg_send, sel, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSControlStateValueOff, NSControlStateValueOn, NSEventMask, NSEventType, NSMenu,
    NSMenuItem, NSRunningApplication, NSStatusBar, NSStatusBarButton, NSStatusItem,
    NSVariableStatusItemLength, NSWorkspaceApplicationKey,
};
use objc2_foundation::{ns_string, NSNotification, NSPoint, NSString, NSTimer};

use super::with_state;

define_class!(
    // NSObject has no subclassing requirements. These selectors are called by AppKit on its main thread.
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    pub(super) struct MenuTarget;

    impl MenuTarget {
        #[unsafe(method(statusClicked:))]
        fn status_clicked(&self, button: &NSStatusBarButton) {
            let app = NSApplication::sharedApplication(self.mtm());
            let context = app.currentEvent().is_some_and(|event| event.r#type() == NSEventType::RightMouseUp);
            if context {
                // Do not hold a State borrow inside AppKit's menu tracking loop.
                if let Some(menu) = with_state(|state| state.menu.clone()).flatten() {
                    menu.popup(button);
                }
            } else {
                with_state(|state| state.toggle_language(false));
            }
        }

        #[unsafe(method(selectInputType:))]
        fn input_type(&self, item: &NSMenuItem) {
            with_state(|state| state.select_input_type(item.tag() as i64));
        }

        #[unsafe(method(selectCodeTable:))]
        fn code_table(&self, item: &NSMenuItem) {
            with_state(|state| state.select_code_table(item.tag() as i64));
        }

        #[unsafe(method(toggleSpelling:))]
        fn spelling(&self, _: &NSMenuItem) {
            with_state(|state| state.toggle_spelling());
        }

        #[unsafe(method(quitNeoKey:))]
        fn quit(&self, _: &NSMenuItem) {
            with_state(|state| state.flush_preferences());
            NSApplication::sharedApplication(self.mtm()).terminate(None);
        }

        #[unsafe(method(watchdog:))]
        fn watchdog(&self, _: &NSTimer) {
            with_state(|state| state.watchdog());
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
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        // NSObject's inherited init leaves this ivar-free instance initialized.
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
    _status: Retained<NSStatusItem>,
    button: Retained<NSStatusBarButton>,
    menu: Retained<NSMenu>,
    input_types: Vec<Retained<NSMenuItem>>,
    tables: Vec<Retained<NSMenuItem>>,
    spelling: Retained<NSMenuItem>,
}

impl StatusMenu {
    pub fn new(mtm: MainThreadMarker, target: &MenuTarget) -> Result<Self, &'static str> {
        let status =
            NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);
        let button = status
            .button(mtm)
            .ok_or("Cannot create NeoKeyRust menu bar button")?;
        unsafe {
            button.setTarget(Some(target));
            button.setAction(Some(sel!(statusClicked:)));
        }
        button.sendActionOn(NSEventMask::LeftMouseUp | NSEventMask::RightMouseUp);
        button.setToolTip(Some(ns_string!(
            "NeoKeyRust. Click to switch language. Right-click for settings."
        )));

        let menu = NSMenu::new(mtm);
        let input_types = add_submenu(
            mtm,
            &menu,
            target,
            ns_string!("Input Type"),
            sel!(selectInputType:),
            &[
                ns_string!("Telex"),
                ns_string!("VNI"),
                ns_string!("Simple Telex 1"),
                ns_string!("Simple Telex 2"),
            ],
        );
        let tables = add_submenu(
            mtm,
            &menu,
            target,
            ns_string!("Code Table"),
            sel!(selectCodeTable:),
            &[
                ns_string!("Unicode"),
                ns_string!("TCVN3"),
                ns_string!("VNI-Windows"),
                ns_string!("Compound Unicode"),
                ns_string!("Vietnamese CP1258"),
            ],
        );
        let spelling = add_item(
            mtm,
            &menu,
            target,
            ns_string!("Spell Check"),
            sel!(toggleSpelling:),
        );
        menu.addItem(&NSMenuItem::separatorItem(mtm));
        add_item(
            mtm,
            &menu,
            target,
            ns_string!("Quit NeoKeyRust"),
            sel!(quitNeoKey:),
        );
        Ok(Self {
            _status: status,
            button,
            menu,
            input_types,
            tables,
            spelling,
        })
    }

    pub fn refresh(&self, settings: &Settings, spelling: bool) {
        self.button.setTitle(if settings.vLanguage != 0 {
            ns_string!("V")
        } else {
            ns_string!("E")
        });
        for (items, selected) in [
            (&self.input_types, settings.vInputType),
            (&self.tables, settings.vCodeTable),
        ] {
            for item in items {
                item.setState(checkmark(item.tag() as i64 == selected));
            }
        }
        self.spelling.setState(checkmark(spelling));
    }

    fn popup(&self, button: &NSStatusBarButton) {
        let bounds = button.bounds();
        let y = if button.isFlipped() {
            bounds.size.height
        } else {
            0.0
        };
        self.menu.popUpMenuPositioningItem_atLocation_inView(
            None,
            NSPoint::new(0.0, y),
            Some(button),
        );
    }
}

fn checkmark(checked: bool) -> isize {
    if checked {
        NSControlStateValueOn
    } else {
        NSControlStateValueOff
    }
}

fn add_submenu(
    mtm: MainThreadMarker,
    menu: &NSMenu,
    target: &MenuTarget,
    title: &NSString,
    action: Sel,
    choices: &[&NSString],
) -> Vec<Retained<NSMenuItem>> {
    let item = NSMenuItem::new(mtm);
    item.setTitle(title);
    menu.addItem(&item);
    let submenu = NSMenu::initWithTitle(NSMenu::alloc(mtm), title);
    item.setSubmenu(Some(&submenu));
    choices
        .iter()
        .enumerate()
        .map(|(index, title)| {
            let choice = add_item(mtm, &submenu, target, title, action);
            choice.setTag(index as isize);
            choice
        })
        .collect()
}

fn add_item(
    mtm: MainThreadMarker,
    menu: &NSMenu,
    target: &MenuTarget,
    title: &NSString,
    action: Sel,
) -> Retained<NSMenuItem> {
    // The action names a MenuTarget selector with an NSMenuItem sender.
    let item = unsafe {
        let item = NSMenuItem::initWithTitle_action_keyEquivalent(
            NSMenuItem::alloc(mtm),
            title,
            Some(action),
            ns_string!(""),
        );
        item.setTarget(Some(target));
        item
    };
    menu.addItem(&item);
    item
}
