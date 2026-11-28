//! Shared AppKit styling helpers for the control panel and settings window.

use objc2::rc::Retained;
use objc2::{define_class, msg_send, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSBezelStyle, NSBox, NSBoxType, NSButton, NSColor, NSControlSize, NSControlStateValueOff,
    NSControlStateValueOn, NSFont, NSFontWeightRegular, NSFontWeightSemibold, NSImage,
    NSImageNameApplicationIcon, NSImageScaling, NSImageView, NSSwitch, NSTextField,
    NSTitlePosition, NSView,
};
use objc2_foundation::{ns_string, NSPoint, NSRect, NSSize, NSString};

pub const ACCENT: (f64, f64, f64) = (
    0x25 as f64 / 255.0,
    0x63 as f64 / 255.0,
    0xEB as f64 / 255.0,
);
pub const PANEL_WIDTH: f64 = 380.0;
pub const PANEL_HEIGHT: f64 = 640.0;
pub const SETTINGS_WIDTH: f64 = 880.0;
pub const SETTINGS_HEIGHT: f64 = 640.0;
pub const SIDEBAR_WIDTH: f64 = 220.0;
pub const ONBOARDING_WIDTH: f64 = 720.0;
pub const ONBOARDING_HEIGHT: f64 = 560.0;

define_class!(
    #[unsafe(super = NSView)]
    #[thread_kind = MainThreadOnly]
    pub struct FlippedView;

    impl FlippedView {
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool {
            true
        }
    }
);

impl FlippedView {
    pub fn new(mtm: MainThreadMarker, frame: NSRect) -> Retained<Self> {
        // NSView's designated initializer; ivar-free subclass.
        unsafe { msg_send![super(Self::alloc(mtm).set_ivars(())), initWithFrame: frame] }
    }
}

pub fn accent_color() -> Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(ACCENT.0, ACCENT.1, ACCENT.2, 1.0)
}

pub fn soft_accent_fill() -> Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(0xEE as f64 / 255.0, 0xF3 as f64 / 255.0, 1.0, 1.0)
}

pub fn sidebar_fill() -> Retained<NSColor> {
    NSColor::colorWithSRGBRed_green_blue_alpha(
        0xF5 as f64 / 255.0,
        0xF5 as f64 / 255.0,
        0xF7 as f64 / 255.0,
        1.0,
    )
}

pub fn semibold(size: f64) -> Retained<NSFont> {
    // NSFontWeight* are AppKit extern statics.
    unsafe { NSFont::systemFontOfSize_weight(size, NSFontWeightSemibold) }
}

pub fn regular(size: f64) -> Retained<NSFont> {
    unsafe { NSFont::systemFontOfSize_weight(size, NSFontWeightRegular) }
}

pub fn bold(size: f64) -> Retained<NSFont> {
    NSFont::boldSystemFontOfSize(size)
}

pub fn ns(text: &str) -> Retained<NSString> {
    NSString::from_str(text)
}

pub fn label(
    mtm: MainThreadMarker,
    text: &str,
    size: f64,
    secondary: bool,
) -> Retained<NSTextField> {
    let field = NSTextField::labelWithString(&ns(text), mtm);
    field.setFont(Some(&regular(size)));
    let color = if secondary {
        NSColor::secondaryLabelColor()
    } else {
        NSColor::labelColor()
    };
    field.setTextColor(Some(&color));
    field
}

pub fn title_label(mtm: MainThreadMarker, text: &str, size: f64) -> Retained<NSTextField> {
    let field = NSTextField::labelWithString(&ns(text), mtm);
    field.setFont(Some(&bold(size)));
    field.setTextColor(Some(&NSColor::labelColor()));
    field
}

pub fn section_header(mtm: MainThreadMarker, text: &str) -> Retained<NSTextField> {
    let field = NSTextField::labelWithString(&ns(text), mtm);
    field.setFont(Some(&semibold(12.0)));
    field.setTextColor(Some(&NSColor::secondaryLabelColor()));
    field
}

pub fn app_icon_view(mtm: MainThreadMarker, size: f64) -> Retained<NSImageView> {
    let view = NSImageView::new(mtm);
    view.setFrameSize(NSSize::new(size, size));
    view.setImageScaling(NSImageScaling::ScaleProportionallyUpOrDown);
    let image = unsafe { NSImage::imageNamed(NSImageNameApplicationIcon) }
        .or_else(|| NSImage::imageNamed(ns_string!("AppIcon")));
    if let Some(image) = image {
        view.setImage(Some(&image));
    }
    view
}

pub fn rounded_box(
    mtm: MainThreadMarker,
    frame: NSRect,
    fill: &NSColor,
    radius: f64,
) -> Retained<NSBox> {
    let box_ = NSBox::new(mtm);
    box_.setBoxType(NSBoxType::Custom);
    box_.setBorderWidth(0.0);
    box_.setCornerRadius(radius);
    box_.setFillColor(fill);
    box_.setTitlePosition(NSTitlePosition::NoTitle);
    box_.setFrame(frame);
    box_
}

pub fn make_switch(mtm: MainThreadMarker, tag: isize, on: bool) -> Retained<NSSwitch> {
    let switch = NSSwitch::new(mtm);
    switch.setControlSize(NSControlSize::Regular);
    switch.setTag(tag);
    set_switch(&switch, on);
    switch
}

pub fn set_switch(switch: &NSSwitch, on: bool) {
    switch.setState(if on {
        NSControlStateValueOn
    } else {
        NSControlStateValueOff
    });
}

pub fn switch_is_on(switch: &NSSwitch) -> bool {
    switch.state() == NSControlStateValueOn
}

pub fn make_button(mtm: MainThreadMarker, title: &str, tag: isize) -> Retained<NSButton> {
    let button = NSButton::new(mtm);
    button.setTitle(&ns(title));
    button.setTag(tag);
    button.setBordered(true);
    button.setBezelStyle(NSBezelStyle::Push);
    button
}

pub fn make_quiet_button(mtm: MainThreadMarker, title: &str, tag: isize) -> Retained<NSButton> {
    let button = NSButton::new(mtm);
    button.setTitle(&ns(title));
    button.setTag(tag);
    button.setBordered(false);
    button.setBezelStyle(NSBezelStyle::Badge);
    button
}

pub fn place(view: &NSView, x: f64, y: f64, w: f64, h: f64) {
    view.setFrame(NSRect::new(NSPoint::new(x, y), NSSize::new(w, h)));
}
