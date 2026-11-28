//! Theo ứng dụng: the smart-switch master toggle (moved here from the control
//! panel) plus the per-app language memory list.
//!
//! Rows show the app's display name + icon (resolved once and cached), not the
//! raw bundle id. Entries that don't resolve to an installed `.app`, plus a few
//! system agents, are hidden from the list only — stored data is untouched and
//! each row keeps its ORIGINAL index as its action tag so the handler in
//! `macos::handle_ui_action` still maps back to the right bundle.

use std::cell::RefCell;
use std::collections::HashMap;

use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{
    NSImage, NSImageScaling, NSImageView, NSScrollView, NSScrollerStyle, NSSwitch, NSView,
    NSWorkspace,
};
use objc2_foundation::{NSFileManager, NSPoint, NSRect, NSSize};

use super::style::*;
use super::tag;
use super::UiSnapshot;
use crate::macos::status_menu::MenuTarget;

/// Shared by the master row's label, hint line and switch. `vRememberCode`
/// defaults to on, so the stored per-app selection includes the code table.
const MASTER_TOOLTIP: &str =
    "Tự nhớ ngôn ngữ và bảng mã dùng gần nhất trong mỗi ứng dụng, rồi tự dùng lại \
     khi quay về ứng dụng đó.";
/// Explains the "chế độ tương thích" marker on launcher rows.
const COMPAT_TOOLTIP: &str =
    "Chế độ tương thích: ô tìm kiếm của Spotlight, Alfred, Raycast được xử lý bằng cách riêng \
     để gõ được tiếng Việt.";

/// Resolved once per bundle id and reused across refreshes.
struct CachedApp {
    name: String,
    icon: Option<Retained<NSImage>>,
    visible: bool,
}

pub struct AppsSection {
    view: Retained<FlippedView>,
    master_switch: Retained<NSSwitch>,
    list: Retained<FlippedView>,
    cache: RefCell<HashMap<String, CachedApp>>,
}

impl AppsSection {
    pub fn build(mtm: MainThreadMarker, target: &MenuTarget, w: f64, h: f64) -> Self {
        let view = FlippedView::new(mtm, NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(w, h)));
        let title = section_title(mtm, "Theo ứng dụng");
        place(&title, MARGIN, 18.0, w - 2.0 * MARGIN, 26.0);
        view.addSubview(&title);

        let card_w = w - 2.0 * MARGIN;

        // Nhớ theo ứng dụng.
        let master_card = card(mtm, MARGIN, 58.0, card_w, ROW_HEIGHT + 12.0);
        let master_rect = row_with_hint_tooltip(
            mtm,
            &master_card,
            6.0,
            "Nhớ ngôn ngữ theo ứng dụng",
            "Tự dùng lại ngôn ngữ gần nhất trong mỗi ứng dụng.",
            MASTER_TOOLTIP,
            NSSize::new(switch_w(mtm), switch_h(mtm)),
        );
        let master_switch = make_switch(mtm, tag::SMART_SWITCH, false);
        master_switch.setToolTip(Some(&ns(MASTER_TOOLTIP)));
        add_control(&master_card, &master_switch, master_rect);
        unsafe {
            master_switch.setTarget(Some(target));
            master_switch.setAction(Some(sel!(uiAction:)));
        }
        view.addSubview(&master_card);

        // The app list lives in one card and scrolls when long.
        let list_top = 58.0 + ROW_HEIGHT + 12.0 + 16.0;
        let list_h = h - list_top - MARGIN;
        let list_card = card(mtm, MARGIN, list_top, card_w, list_h);
        let scroll = NSScrollView::new(mtm);
        place(&scroll, 0.0, 0.0, card_w, list_h);
        scroll.setHasVerticalScroller(true);
        scroll.setDrawsBackground(false);
        scroll.setScrollerStyle(NSScrollerStyle::Overlay);
        let list = FlippedView::new(
            mtm,
            NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(card_w, 1.0)),
        );
        scroll.setDocumentView(Some(&list));
        list_card.addSubview(&scroll);
        view.addSubview(&list_card);

        Self {
            view,
            master_switch,
            list,
            cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }

    pub fn refresh_apps(
        &self,
        mtm: MainThreadMarker,
        target: &MenuTarget,
        rows: &[(String, i8, bool)],
        _dark: bool,
    ) {
        // subviews() returns a snapshot copy, so remove each view it lists.
        for view in self.list.subviews() {
            view.removeFromSuperview();
        }
        let width = self.list.bounds().size.width.max(1.0);
        let mut cache = self.cache.borrow_mut();
        let mut y = 0.0;
        let mut shown = 0usize;
        for (i, (bundle, stored, compat)) in rows.iter().enumerate() {
            let app = cache
                .entry(bundle.clone())
                .or_insert_with(|| resolve_app(bundle));
            if !app.visible {
                continue;
            }

            if let Some(icon) = &app.icon {
                let icon_view = NSImageView::new(mtm);
                icon_view.setImageScaling(NSImageScaling::ScaleProportionallyUpOrDown);
                icon_view.setImage(Some(icon));
                place(&icon_view, 12.0, y + 12.0, 20.0, 20.0);
                self.list.addSubview(&icon_view);
            }
            let name = label(mtm, &app.name, 13.0, false);
            let row_tooltip = ns(&format!(
                "Ngôn ngữ gõ sẽ dùng trong {} khi bật Nhớ ngôn ngữ theo ứng dụng.",
                app.name
            ));
            name.setToolTip(Some(&row_tooltip));
            place(&name, 40.0, y + 5.0, width - 180.0, 18.0);
            self.list.addSubview(&name);
            let mut secondary = bundle.clone();
            if *compat {
                secondary.push_str(" · chế độ tương thích");
            }
            let sub = label(mtm, &secondary, 11.0, true);
            if *compat {
                sub.setToolTip(Some(&ns(COMPAT_TOOLTIP)));
            }
            place(&sub, 40.0, y + 24.0, width - 180.0, 14.0);
            self.list.addSubview(&sub);

            // Small system-default segment (no accent bezel); tag keeps the
            // ORIGINAL index so the handler resolves the right bundle.
            let vi = neokey_engine::SmartSwitch::language(*stored) != 0;
            let segment = make_lang_segment_small(mtm, tag::APP_LANG + i as isize);
            segment.setSelectedSegment(if vi { 0 } else { 1 });
            segment.setToolTip(Some(&row_tooltip));
            segment
                .setToolTip_forSegment(Some(&ns(&format!("Gõ tiếng Việt trong {}.", app.name))), 0);
            segment
                .setToolTip_forSegment(Some(&ns(&format!("Gõ tiếng Anh trong {}.", app.name))), 1);
            place(&segment, width - 132.0, y + 10.0, 120.0, 24.0);
            unsafe {
                segment.setTarget(Some(target));
                segment.setAction(Some(sel!(uiAction:)));
            }
            self.list.addSubview(&segment);

            if shown > 0 {
                let separator = separator_box(
                    mtm,
                    NSRect::new(NSPoint::new(40.0, y - 0.5), NSSize::new(width - 52.0, 1.0)),
                );
                self.list.addSubview(&separator);
            }
            y += ROW_HEIGHT;
            shown += 1;
        }
        self.list
            .setFrameSize(NSSize::new(width, y.max(ROW_HEIGHT)));
    }

    pub fn refresh(&self, snap: &UiSnapshot) {
        set_switch(&self.master_switch, snap.settings.vUseSmartSwitchKey != 0);
    }
}

/// Obvious system agents that resolve to a `.app` but aren't user apps.
fn is_system_agent(bundle: &str) -> bool {
    matches!(
        bundle,
        "com.apple.loginwindow" | "com.apple.UserNotificationCenter"
    ) || bundle.starts_with("com.apple.accessibility.")
}

/// Resolve a bundle id to a display name + icon, or mark it hidden. Lookups hit
/// NSWorkspace only once per bundle (the result is cached by the caller).
fn resolve_app(bundle: &str) -> CachedApp {
    let hidden = CachedApp {
        name: bundle.to_string(),
        icon: None,
        visible: false,
    };
    if is_system_agent(bundle) {
        return hidden;
    }
    let workspace = NSWorkspace::sharedWorkspace();
    let Some(url) = workspace.URLForApplicationWithBundleIdentifier(&ns(bundle)) else {
        return hidden;
    };
    let Some(path) = url.path() else {
        return hidden;
    };
    let path_str = path.to_string();
    let is_app = std::path::Path::new(&path_str)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("app"));
    if !is_app {
        return hidden;
    }
    let manager = NSFileManager::defaultManager();
    if !manager.fileExistsAtPath(&path) {
        return hidden;
    }
    let name = manager.displayNameAtPath(&path).to_string();
    let icon = workspace.iconForFile(&path);
    icon.setSize(NSSize::new(20.0, 20.0));
    CachedApp {
        name,
        icon: Some(icon),
        visible: true,
    }
}
