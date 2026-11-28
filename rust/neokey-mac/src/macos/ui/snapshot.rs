//! Debug-only UI snapshot renderer.
//!
//! Enabled by `NEOKEY_UI_SNAPSHOT=<dir>` at launch (wired in `macos::run`). For
//! each of the light and dark appearances it refreshes the UI and renders the
//! panel root view plus every settings section to PNG, then the process exits
//! with status 0. It never installs the event tap and never shows onboarding.
//! This is a development aid, not part of the shipping UI.

use objc2::runtime::AnyObject;
use objc2::MainThreadMarker;
use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization, NSAppearanceNameAqua, NSAppearanceNameDarkAqua,
    NSApplication, NSBitmapImageFileType, NSView,
};
use objc2_foundation::{NSDictionary, NSString, NSURL};

use super::settings_window::SettingsSection;
use super::style::ns;
use super::Ui;

/// Render the panel and all settings sections in light + dark, then return.
/// The caller exits the process afterwards. `refresh` re-resolves the UI for
/// the current appearance (it borrows the app state).
pub(crate) fn run(dir: &str, ui: &Ui, refresh: impl Fn()) {
    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    let _ = std::fs::create_dir_all(dir);

    // NSAppearanceName* are AppKit extern statics.
    let appearances = [
        (unsafe { NSAppearanceNameAqua }, "light"),
        (unsafe { NSAppearanceNameDarkAqua }, "dark"),
    ];
    for (name, suffix) in appearances {
        let Some(appearance) = NSAppearance::appearanceNamed(name) else {
            continue;
        };
        app.setAppearance(Some(&appearance));
        refresh();

        if let Some(panel_view) = ui.panel.snapshot_view() {
            panel_view.setAppearance(Some(&appearance));
            render(&panel_view, &format!("{dir}/panel-{suffix}.png"));
        }
        if let Some(root) = ui.settings.snapshot_view() {
            root.setAppearance(Some(&appearance));
            for section in SettingsSection::ALL {
                ui.settings.select(section);
                render(
                    &root,
                    &format!("{dir}/settings-{}-{suffix}.png", section.slug()),
                );
            }
        }
    }
}

/// Cache the view's display into a bitmap and write it out as PNG.
fn render(view: &NSView, path: &str) {
    let bounds = view.bounds();
    let Some(rep) = view.bitmapImageRepForCachingDisplayInRect(bounds) else {
        return;
    };
    view.cacheDisplayInRect_toBitmapImageRep(bounds, &rep);
    let properties = NSDictionary::<NSString, AnyObject>::dictionary();
    // Safety: `properties` is an empty dictionary of the expected key/value types.
    let data =
        unsafe { rep.representationUsingType_properties(NSBitmapImageFileType::PNG, &properties) };
    let Some(data) = data else {
        return;
    };
    let url = NSURL::fileURLWithPath(&ns(path));
    let _ = data.writeToURL_atomically(&url, true);
}
