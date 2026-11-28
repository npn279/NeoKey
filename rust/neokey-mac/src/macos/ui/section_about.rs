use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{NSButton, NSSwitch, NSView};
use objc2_foundation::{NSBundle, NSPoint, NSRect, NSSize};

use super::style::*;
use super::{tag, UiSnapshot};
use crate::macos::status_menu::MenuTarget;

pub struct AboutSection {
    view: Retained<FlippedView>,
    links: Vec<Retained<NSButton>>,
    update: Retained<NSButton>,
    startup_update: Retained<NSSwitch>,
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
        repo.setToolTip(Some(&ns(
            "Mở kho mã nguồn NeoKey trên GitHub (npn279/NeoKey).",
        )));
        place(&repo, center - 170.0, 176.0, 160.0, 28.0);
        let issues = make_link_button(mtm, "Báo lỗi", tag::LINK + 1);
        issues.setToolTip(Some(&ns("Mở trang báo lỗi của NeoKey trên GitHub.")));
        place(&issues, center + 10.0, 176.0, 120.0, 28.0);
        for btn in [&repo, &issues] {
            unsafe {
                btn.setTarget(Some(target));
                btn.setAction(Some(sel!(uiAction:)));
            }
        }

        let more_links = [
            ("Bản phát hành", 2),
            ("Trang OpenKey", 3),
            ("Facebook", 4),
            ("Email", 5),
            ("Mã nguồn OpenKey", 6),
        ];
        let upstream = links_card(mtm, target, card_w, &more_links);
        view.addSubview(&upstream.0);
        let update_card = super::style::card(mtm, MARGIN, 396.0, card_w, 112.0);
        let update = make_button(mtm, "Kiểm tra cập nhật", tag::UPDATE);
        update.setToolTip(Some(&ns("Kiểm tra phiên bản mới trên GitHub.")));
        place(&update, CARD_PAD, 13.0, 176.0, 30.0);
        update_card.addSubview(&update);
        row_separator(mtm, &update_card, 55.0);
        let tip = "Tự kiểm tra phiên bản mới khi mở NeoKey.";
        let rect = row_with_tooltip(
            mtm,
            &update_card,
            60.0,
            "Kiểm tra cập nhật khi khởi động",
            tip,
            switch_w(mtm),
            switch_h(mtm),
        );
        let startup_update = make_switch(mtm, tag::STARTUP_UPDATE, true);
        startup_update.setToolTip(Some(&ns(tip)));
        add_control(&update_card, &startup_update, rect);
        for control in [&update as &objc2_app_kit::NSControl, &startup_update] {
            unsafe {
                control.setTarget(Some(target));
                control.setAction(Some(sel!(uiAction:)));
            }
        }
        view.addSubview(&update_card);

        let ver = NSBundle::mainBundle()
            .objectForInfoDictionaryKey(objc2_foundation::ns_string!("CFBundleShortVersionString"))
            .and_then(|s| s.downcast::<objc2_foundation::NSString>().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "0.1.0".into());
        let display =
            crate::macos::updates::bundle_version(objc2_foundation::ns_string!("CFBundleVersion"))
                .map_or_else(
                    || format!("Phiên bản {ver}"),
                    |build| format!("Phiên bản {ver} (build {build})"),
                );
        version.setStringValue(&ns(&display));

        if let Some(date) =
            crate::macos::updates::bundle_version(objc2_foundation::ns_string!("NeoKeyBuildDate"))
        {
            let build_date = centered_label(mtm, &format!("Ngày cập nhật {date}"), 11.0, true);
            place(&build_date, center - 120.0, 155.0, 240.0, 16.0);
            card.addSubview(&build_date);
        }
        for item in [&icon as &NSView, &name, &version, &repo, &issues] {
            card.addSubview(item);
        }
        view.addSubview(&card);
        Self {
            view,
            links: [vec![repo, issues], upstream.1].concat(),
            update,
            startup_update,
        }
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }

    pub fn refresh(&self, snap: &UiSnapshot, dark: bool) {
        self.update.setEnabled(!snap.checking_updates);
        self.update.setTitle(&ns(if snap.checking_updates {
            "Đang kiểm tra…"
        } else {
            "Kiểm tra cập nhật"
        }));
        set_switch(&self.startup_update, snap.update_on_startup);
        let accent = accent_color(dark);
        for link in &self.links {
            link.setContentTintColor(Some(&accent));
        }
    }
}

fn links_card(
    mtm: MainThreadMarker,
    target: &MenuTarget,
    width: f64,
    links: &[(&str, isize)],
) -> (Retained<FlippedView>, Vec<Retained<NSButton>>) {
    let view = card(mtm, MARGIN, 304.0, width, 76.0);
    let buttons = links
        .iter()
        .enumerate()
        .map(|(index, (text, offset))| {
            let button = make_link_button(mtm, text, tag::LINK + offset);
            let x = CARD_PAD + (index % 3) as f64 * 160.0;
            let y = 8.0 + (index / 3) as f64 * 30.0;
            place(&button, x, y, 156.0, 28.0);
            unsafe {
                button.setTarget(Some(target));
                button.setAction(Some(sel!(uiAction:)));
            }
            view.addSubview(&button);
            button
        })
        .collect();
    (view, buttons)
}
