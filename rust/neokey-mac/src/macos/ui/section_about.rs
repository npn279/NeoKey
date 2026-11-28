use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::NSView;
use objc2_foundation::NSBundle;

use super::style::*;
use super::tag;
use super::UiSnapshot;
use crate::macos::status_menu::MenuTarget;

pub struct AboutSection {
    view: Retained<FlippedView>,
}

impl AboutSection {
    pub fn build(mtm: MainThreadMarker, target: &MenuTarget, w: f64, h: f64) -> Self {
        let view = FlippedView::new(
            mtm,
            objc2_foundation::NSRect::new(
                objc2_foundation::NSPoint::new(0.0, 0.0),
                objc2_foundation::NSSize::new(w, h),
            ),
        );
        let icon = app_icon_view(mtm, 72.0);
        place(&icon, w / 2.0 - 36.0, 40.0, 72.0, 72.0);
        let title = title_label(mtm, "NeoKey", 26.0);
        place(&title, w / 2.0 - 60.0, 130.0, 120.0, 32.0);
        let version = label(mtm, "", 14.0, true);
        place(&version, w / 2.0 - 120.0, 170.0, 240.0, 20.0);

        let repo = make_button(mtm, "Mã nguồn trên GitHub", tag::LINK);
        let issues = make_button(mtm, "Báo lỗi", tag::LINK + 1);
        place(&repo, w / 2.0 - 180.0, 220.0, 160.0, 32.0);
        place(&issues, w / 2.0 + 20.0, 220.0, 120.0, 32.0);
        for btn in [&repo, &issues] {
            unsafe {
                btn.setTarget(Some(target));
                btn.setAction(Some(sel!(uiAction:)));
            }
        }

        let ver = NSBundle::mainBundle()
            .objectForInfoDictionaryKey(objc2_foundation::ns_string!("CFBundleShortVersionString"))
            .and_then(|s| s.downcast::<objc2_foundation::NSString>().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "0.1.0".into());
        version.setStringValue(&ns(&format!("Phiên bản {ver}")));

        view.addSubview(&icon);
        view.addSubview(&title);
        view.addSubview(&version);
        view.addSubview(&repo);
        view.addSubview(&issues);
        Self { view }
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }

    pub fn refresh(&self, _snap: &UiSnapshot) {}
}
