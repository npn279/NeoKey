use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{NSScrollView, NSView};

use super::style::*;
use super::tag;
use super::UiSnapshot;
use crate::macos::status_menu::MenuTarget;

pub struct AppsSection {
    view: Retained<FlippedView>,
    list: Retained<FlippedView>,
}

impl AppsSection {
    pub fn build(mtm: MainThreadMarker, target: &MenuTarget, w: f64, h: f64) -> Self {
        let view = FlippedView::new(
            mtm,
            objc2_foundation::NSRect::new(
                objc2_foundation::NSPoint::new(0.0, 0.0),
                objc2_foundation::NSSize::new(w, h),
            ),
        );
        let title = title_label(mtm, "Theo ứng dụng", 22.0);
        place(&title, 24.0, 16.0, 260.0, 30.0);
        view.addSubview(&title);
        let scroll = NSScrollView::new(mtm);
        place(&scroll, 24.0, 60.0, w - 48.0, h - 80.0);
        let list = FlippedView::new(
            mtm,
            objc2_foundation::NSRect::new(
                objc2_foundation::NSPoint::new(0.0, 0.0),
                objc2_foundation::NSSize::new(w - 48.0, 400.0),
            ),
        );
        scroll.setDocumentView(Some(&list));
        view.addSubview(&scroll);
        let _ = (target, tag::APP_LANG);
        Self { view, list }
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }

    pub fn refresh_apps(
        &self,
        mtm: MainThreadMarker,
        target: &MenuTarget,
        rows: &[(String, i8, bool)],
    ) {
        // subviews() returns a snapshot copy, so remove each view it lists.
        for view in self.list.subviews() {
            view.removeFromSuperview();
        }
        for (i, (bundle, stored, compat)) in rows.iter().enumerate() {
            let y = i as f64 * 52.0;
            let lab = label(mtm, bundle, 13.0, false);
            place(&lab, 8.0, y + 8.0, 320.0, 18.0);
            if *compat {
                let badge = label(mtm, "Chế độ tương thích", 11.0, true);
                place(&badge, 8.0, y + 28.0, 280.0, 16.0);
                self.list.addSubview(&badge);
            }
            let vi_on = neokey_engine::SmartSwitch::language(*stored) != 0;
            let vi = make_button(mtm, "Việt", tag::APP_LANG + (i * 2) as isize);
            let en = make_button(mtm, "English", tag::APP_LANG + (i * 2 + 1) as isize);
            vi.setBordered(!vi_on);
            en.setBordered(vi_on);
            if vi_on {
                vi.setContentTintColor(Some(&accent_color()));
            } else {
                en.setContentTintColor(Some(&accent_color()));
            }
            place(&vi, 360.0, y + 10.0, 72.0, 28.0);
            place(&en, 438.0, y + 10.0, 80.0, 28.0);
            unsafe {
                vi.setTarget(Some(target));
                vi.setAction(Some(sel!(uiAction:)));
                en.setTarget(Some(target));
                en.setAction(Some(sel!(uiAction:)));
            }
            self.list.addSubview(&lab);
            self.list.addSubview(&vi);
            self.list.addSubview(&en);
        }
        self.list.setFrameSize(objc2_foundation::NSSize::new(
            self.list.bounds().size.width,
            rows.len() as f64 * 52.0,
        ));
    }

    pub fn refresh(&self, _snap: &UiSnapshot) {}
}
