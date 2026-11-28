use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{NSButton, NSView};
use objc2_foundation::{NSBundle, NSPoint, NSRect, NSSize};

use super::style::*;
use super::tag;
use crate::macos::status_menu::MenuTarget;

pub struct AboutSection {
    view: Retained<FlippedView>,
    links: Vec<Retained<NSButton>>,
}

impl AboutSection {
    pub fn build(mtm: MainThreadMarker, target: &MenuTarget, w: f64, h: f64) -> Self {
        let view = FlippedView::new(mtm, NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(w, h)));
        let title = section_title(mtm, "Giới thiệu");
        place(&title, MARGIN, 18.0, w - 2.0 * MARGIN, 26.0);
        view.addSubview(&title);

        let card_w = w - 2.0 * MARGIN;
        let card = card(mtm, MARGIN, 58.0, card_w, 230.0);
        let center = card_w / 2.0;

        let icon = app_icon_view(mtm, 64.0);
        place(&icon, center - 32.0, 24.0, 64.0, 64.0);
        let name = centered_label(mtm, "NeoKey", 22.0, false);
        place(&name, center - 100.0, 100.0, 200.0, 28.0);
        let version = centered_label(mtm, "", 13.0, true);
        place(&version, center - 120.0, 134.0, 240.0, 18.0);

        let repo = make_link_button(mtm, "Mã nguồn trên GitHub", tag::LINK);
        place(&repo, center - 170.0, 176.0, 160.0, 28.0);
        let issues = make_link_button(mtm, "Báo lỗi", tag::LINK + 1);
        place(&issues, center + 10.0, 176.0, 120.0, 28.0);
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

        for item in [&icon as &NSView, &name, &version, &repo, &issues] {
            card.addSubview(item);
        }
        view.addSubview(&card);
        Self {
            view,
            links: vec![repo, issues],
        }
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }

    pub fn refresh(&self, dark: bool) {
        let accent = accent_color(dark);
        for link in &self.links {
            link.setContentTintColor(Some(&accent));
        }
    }
}
