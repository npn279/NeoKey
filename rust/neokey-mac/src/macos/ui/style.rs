//! Shared AppKit styling helpers for the control panel and settings window.
//!
//! Design rules:
//! - One accent color (NeoKey blue, lighter in dark mode); everything else is a
//!   semantic system color so light/dark mode both stay readable.
//! - Frame-based layout inside flipped views, driven by [`Stack`], [`card`] and
//!   [`row_with_tooltip`] so spacing math lives only here.

use objc2::rc::Retained;
use objc2::{define_class, msg_send, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::NSTextAlignment;
use objc2_app_kit::{
    NSAppearance, NSApplication, NSBezierPath, NSBox, NSBoxType, NSButton, NSCellImagePosition,
    NSColor, NSControlSize, NSControlStateValueOff, NSControlStateValueOn, NSFont,
    NSFontWeightRegular, NSFontWeightSemibold, NSImage, NSImageNameApplicationIcon, NSImageScaling,
    NSImageView, NSSegmentDistribution, NSSegmentSwitchTracking, NSSegmentedControl, NSSwitch,
    NSTextField, NSTitlePosition, NSView,
};
use objc2_foundation::{ns_string, NSPoint, NSRect, NSSize, NSString};

// ---- Accent ---------------------------------------------------------------
// Derived from NeoKey's blue app icon. `ACCENT_DARK` is lighter so the single
// accent stays readable on dark fills.

/// `#2563EB` in light mode.
pub const ACCENT_LIGHT: (f64, f64, f64) = (
    0x25 as f64 / 255.0,
    0x63 as f64 / 255.0,
    0xEB as f64 / 255.0,
);
/// `#5B8CFF` in dark mode.
pub const ACCENT_DARK: (f64, f64, f64) = (
    0x5B as f64 / 255.0,
    0x8C as f64 / 255.0,
    0xFF as f64 / 255.0,
);

// ---- Layout ---------------------------------------------------------------

/// Right-click control panel.
pub const PANEL_WIDTH: f64 = 292.0;
pub const PANEL_HEIGHT: f64 = 224.0;
pub const SETTINGS_WIDTH: f64 = 780.0;
pub const SETTINGS_HEIGHT: f64 = 620.0;
pub const SIDEBAR_WIDTH: f64 = 210.0;
pub const ONBOARDING_WIDTH: f64 = 720.0;
pub const ONBOARDING_HEIGHT: f64 = 560.0;

/// Outer margin of cards and lists.
pub const MARGIN: f64 = 20.0;
/// Horizontal padding inside a card.
pub const CARD_PAD: f64 = 16.0;
/// Standard card row height.
pub const ROW_HEIGHT: f64 = 44.0;
pub const CARD_RADIUS: f64 = 10.0;

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

// Opaque panel background: solid `windowBackgroundColor`, rounded to match the
// popover chrome, so labels never float on a translucent vibrant blur.
define_class!(
    #[unsafe(super = NSView)]
    #[thread_kind = MainThreadOnly]
    pub struct SolidView;

    impl SolidView {
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool {
            true
        }

        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _dirty_rect: NSRect) {
            let path = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                self.bounds(),
                11.0,
                11.0,
            );
            NSColor::windowBackgroundColor().setFill();
            path.fill();
        }
    }
);

impl FlippedView {
    pub fn new(mtm: MainThreadMarker, frame: NSRect) -> Retained<Self> {
        // NSView's designated initializer; ivar-free subclass.
        unsafe { msg_send![super(Self::alloc(mtm).set_ivars(())), initWithFrame: frame] }
    }
}

impl SolidView {
    pub fn new(mtm: MainThreadMarker, frame: NSRect) -> Retained<Self> {
        unsafe { msg_send![super(Self::alloc(mtm).set_ivars(())), initWithFrame: frame] }
    }
}

// ---- Colors ----------------------------------------------------------------

/// True when `appearance` renders dark content (dark mode or a dark vibrant
/// material). Callers re-resolve the accent on every refresh.
pub fn is_dark(appearance: &NSAppearance) -> bool {
    let name = appearance.name().to_string();
    name.contains("DarkAqua")
        || name.contains("VibrantDark")
        || name == "NSAppearanceNameLightContent"
}

/// Convenience wrapper for the current application appearance.
pub fn app_is_dark() -> bool {
    let Some(mtm) = MainThreadMarker::new() else {
        return false;
    };
    is_dark(&NSApplication::sharedApplication(mtm).effectiveAppearance())
}

pub fn accent_color(dark: bool) -> Retained<NSColor> {
    let (red, green, blue) = if dark { ACCENT_DARK } else { ACCENT_LIGHT };
    NSColor::colorWithSRGBRed_green_blue_alpha(red, green, blue, 1.0)
}

/// Soft tinted fill for selected pills and banners.
pub fn soft_accent_fill(dark: bool) -> Retained<NSColor> {
    let alpha = if dark { 0.25 } else { 0.12 };
    accent_color(dark).colorWithAlphaComponent(alpha)
}

// ---- Fonts and labels ------------------------------------------------------

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

pub fn centered_label(
    mtm: MainThreadMarker,
    text: &str,
    size: f64,
    secondary: bool,
) -> Retained<NSTextField> {
    let field = label(mtm, text, size, secondary);
    field.setAlignment(NSTextAlignment::Center);
    field
}

pub fn title_label(mtm: MainThreadMarker, text: &str, size: f64) -> Retained<NSTextField> {
    let field = NSTextField::labelWithString(&ns(text), mtm);
    field.setFont(Some(&bold(size)));
    field.setTextColor(Some(&NSColor::labelColor()));
    field
}

/// Small secondary header shown above a card.
pub fn section_header(mtm: MainThreadMarker, text: &str) -> Retained<NSTextField> {
    let field = NSTextField::labelWithString(&ns(text), mtm);
    field.setFont(Some(&semibold(12.0)));
    field.setTextColor(Some(&NSColor::secondaryLabelColor()));
    field
}

/// Section title: ~20 pt semibold, top-left, aligned with the cards' left edge.
pub fn section_title(mtm: MainThreadMarker, text: &str) -> Retained<NSTextField> {
    let field = NSTextField::labelWithString(&ns(text), mtm);
    field.setFont(Some(&semibold(20.0)));
    field.setTextColor(Some(&NSColor::labelColor()));
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

// ---- Boxes, separators, cards ----------------------------------------------

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

/// One-point separator line.
pub fn separator_box(mtm: MainThreadMarker, frame: NSRect) -> Retained<NSBox> {
    rounded_box(mtm, frame, &NSColor::separatorColor(), 0.0)
}

/// Top-down y cursor for frame-based layout inside flipped views.
pub struct Stack {
    y: f64,
}

impl Stack {
    pub fn new(top: f64) -> Self {
        Self { y: top }
    }

    /// Returns the y where a block of `height` starts, then advances.
    pub fn take(&mut self, height: f64) -> f64 {
        let y = self.y;
        self.y += height;
        y
    }

    pub fn gap(&mut self, gap: f64) {
        self.y += gap;
    }

    /// The y just below everything placed so far.
    pub fn bottom(&self) -> f64 {
        self.y
    }
}

/// Rounded card: a `controlBackgroundColor` group with a hairline
/// `separatorColor` border so it reads against the `windowBackgroundColor`
/// content area. Rows are added as subviews on top of the fill.
pub fn card(mtm: MainThreadMarker, x: f64, y: f64, w: f64, h: f64) -> Retained<FlippedView> {
    let view = FlippedView::new(mtm, NSRect::new(NSPoint::new(x, y), NSSize::new(w, h)));
    let fill = rounded_box(
        mtm,
        NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(w, h)),
        &NSColor::controlBackgroundColor(),
        CARD_RADIUS,
    );
    fill.setBorderWidth(1.0);
    fill.setBorderColor(&NSColor::separatorColor());
    view.addSubview(&fill);
    view
}

/// Places a control inside a card AND adds it as a subview in one step, so a
/// control can never be positioned but left invisible.
pub fn add_control(card: &FlippedView, control: &NSView, frame: NSRect) {
    control.setFrame(frame);
    card.addSubview(control);
}

/// Places `text` as a row label with a tooltip and returns the frame for the
/// row's control (right-aligned inside the card). `y` is the top of a
/// `ROW_HEIGHT` slot. Share `tooltip` with the control so hovering either one
/// explains the same setting.
pub fn row_with_tooltip(
    mtm: MainThreadMarker,
    card: &FlippedView,
    y: f64,
    text: &str,
    tooltip: &str,
    control_w: f64,
    control_h: f64,
) -> NSRect {
    row_impl(
        mtm,
        card,
        y,
        text,
        None,
        Some(tooltip),
        NSSize::new(control_w, control_h),
    )
}

/// Thin separator at the top boundary of the row slot starting at `y`.
pub fn row_separator(mtm: MainThreadMarker, card: &FlippedView, y: f64) {
    let w = card.bounds().size.width;
    let line = separator_box(
        mtm,
        NSRect::new(
            NSPoint::new(CARD_PAD, y - 0.5),
            NSSize::new(w - 2.0 * CARD_PAD, 1.0),
        ),
    );
    card.addSubview(&line);
}

/// A row whose label carries a secondary hint line directly beneath it; the
/// control stays vertically centred against the label block. Hovering the
/// label or the hint line shows `tooltip`. `y` is the top of a `ROW_HEIGHT`
/// slot. Returns the control frame. Takes the control size as one [`NSSize`]
/// to stay within clippy's argument limit.
pub fn row_with_hint_tooltip(
    mtm: MainThreadMarker,
    card: &FlippedView,
    y: f64,
    text: &str,
    hint: &str,
    tooltip: &str,
    control: NSSize,
) -> NSRect {
    row_impl(mtm, card, y, text, Some(hint), Some(tooltip), control)
}

/// Shared layout for the row helpers: adds the label (and the optional hint
/// line beneath it), attaches `tooltip` to every label field so hovering any
/// of them shows it, and returns the control frame.
fn row_impl(
    mtm: MainThreadMarker,
    card: &FlippedView,
    y: f64,
    text: &str,
    hint: Option<&str>,
    tooltip: Option<&str>,
    control: NSSize,
) -> NSRect {
    let w = card.bounds().size.width;
    let label_w = (w - 2.0 * CARD_PAD - control.width - 8.0).max(60.0);
    let text_field = label(mtm, text, 14.0, false);
    let hint_field = hint.map(|hint| label(mtm, hint, 11.0, true));
    if let Some(tooltip) = tooltip {
        let tooltip = ns(tooltip);
        text_field.setToolTip(Some(&tooltip));
        if let Some(hint_field) = &hint_field {
            hint_field.setToolTip(Some(&tooltip));
        }
    }
    if hint.is_some() {
        place(&text_field, CARD_PAD, y + 5.0, label_w, 18.0);
        if let Some(hint_field) = &hint_field {
            place(hint_field, CARD_PAD, y + 24.0, label_w, 14.0);
            card.addSubview(hint_field);
        }
    } else {
        place(
            &text_field,
            CARD_PAD,
            y + (ROW_HEIGHT - 20.0) / 2.0,
            label_w,
            20.0,
        );
    }
    card.addSubview(&text_field);
    if let Some(tooltip) = tooltip {
        super::help::add_help(mtm, card, &text_field, tooltip);
    }
    NSRect::new(
        NSPoint::new(
            w - CARD_PAD - control.width,
            y + (ROW_HEIGHT - control.height) / 2.0,
        ),
        control,
    )
}

// ---- Controls ---------------------------------------------------------------

pub fn make_switch(mtm: MainThreadMarker, tag: isize, on: bool) -> Retained<NSSwitch> {
    let switch = NSSwitch::new(mtm);
    switch.setControlSize(NSControlSize::Regular);
    switch.setTag(tag);
    set_switch(&switch, on);
    switch
}

/// The switch's natural size. Frames smaller than this make AppKit draw the
/// switch past its frame, so rows reserve exactly this much.
fn switch_size(mtm: MainThreadMarker) -> NSSize {
    NSSwitch::new(mtm).fittingSize()
}

pub fn switch_w(mtm: MainThreadMarker) -> f64 {
    switch_size(mtm).width
}

pub fn switch_h(mtm: MainThreadMarker) -> f64 {
    switch_size(mtm).height
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
    button.setBezelStyle(objc2_app_kit::NSBezelStyle::Push);
    button
}

/// Plain borderless text button (panel footer, list actions).
pub fn make_quiet_button(mtm: MainThreadMarker, title: &str, tag: isize) -> Retained<NSButton> {
    let button = NSButton::new(mtm);
    button.setTitle(&ns(title));
    button.setTag(tag);
    button.setBordered(false);
    button.setBezelStyle(objc2_app_kit::NSBezelStyle::Badge);
    button
}

/// Borderless button showing an SF Symbol (falls back to a text glyph if the
/// symbol is unavailable).
pub fn make_icon_button(
    mtm: MainThreadMarker,
    symbol: &str,
    fallback_title: &str,
    tag: isize,
) -> Retained<NSButton> {
    let button = NSButton::new(mtm);
    button.setTag(tag);
    button.setBordered(false);
    button.setBezelStyle(objc2_app_kit::NSBezelStyle::Badge);
    if let Some(image) = symbol_image(symbol) {
        button.setImage(Some(&image));
        button.setImagePosition(NSCellImagePosition::ImageOnly);
    } else {
        button.setTitle(&ns(fallback_title));
    }
    button.setContentTintColor(Some(&NSColor::secondaryLabelColor()));
    button
}

/// A left-aligned sidebar item: a 16 pt SF Symbol (secondary, accent when
/// selected) with the title 8 pt to its right, both vertically centred in a
/// 32 pt row. A transparent button overlays the row to carry the action.
pub struct SidebarItem {
    pub view: Retained<FlippedView>,
    icon: Retained<NSImageView>,
    title: Retained<NSTextField>,
    pub button: Retained<NSButton>,
}

impl SidebarItem {
    pub fn new(mtm: MainThreadMarker, title: &str, symbol: &str, tag: isize, w: f64) -> Self {
        let h = 32.0;
        let view = FlippedView::new(mtm, NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(w, h)));
        let icon = NSImageView::new(mtm);
        icon.setImageScaling(NSImageScaling::ScaleProportionallyUpOrDown);
        if let Some(image) = symbol_image(symbol) {
            icon.setImage(Some(&image));
        }
        icon.setContentTintColor(Some(&NSColor::secondaryLabelColor()));
        place(&icon, 10.0, 8.0, 16.0, 16.0);
        let title_field = label(mtm, title, 13.0, false);
        place(&title_field, 34.0, 8.0, (w - 42.0).max(40.0), 16.0);
        let button = NSButton::new(mtm);
        button.setTag(tag);
        button.setBordered(false);
        button.setTitle(&ns(""));
        place(&button, 0.0, 0.0, w, h);
        view.addSubview(&icon);
        view.addSubview(&title_field);
        view.addSubview(&button);
        Self {
            view,
            icon,
            title: title_field,
            button,
        }
    }

    /// Selected: accent icon + accent title; otherwise secondary icon + plain
    /// label, so there is no stale tint state.
    pub fn set_selected(&self, selected: bool, accent: &Retained<NSColor>) {
        let icon_tint = if selected {
            accent.clone()
        } else {
            NSColor::secondaryLabelColor()
        };
        self.icon.setContentTintColor(Some(&icon_tint));
        let text_color = if selected {
            accent.clone()
        } else {
            NSColor::labelColor()
        };
        self.title.setTextColor(Some(&text_color));
    }
}

/// Borderless text link in the accent color (About section).
pub fn make_link_button(mtm: MainThreadMarker, title: &str, tag: isize) -> Retained<NSButton> {
    let button = make_quiet_button(mtm, title, tag);
    button.setFont(Some(&regular(14.0)));
    button
}

/// Two-segment Việt / English select-one picker.
pub fn make_lang_segment(mtm: MainThreadMarker, tag: isize) -> Retained<NSSegmentedControl> {
    let segment = NSSegmentedControl::new(mtm);
    segment.setSegmentCount(2);
    segment.setLabel_forSegment(&ns("Việt"), 0);
    segment.setLabel_forSegment(&ns("English"), 1);
    segment.setTrackingMode(NSSegmentSwitchTracking::SelectOne);
    segment.setSegmentDistribution(NSSegmentDistribution::FillEqually);
    segment.setTag(tag);
    segment
}

/// Compact variant for dense lists (per-app rows): small control size and the
/// system default selection colour — no custom accent bezel.
pub fn make_lang_segment_small(mtm: MainThreadMarker, tag: isize) -> Retained<NSSegmentedControl> {
    let segment = make_lang_segment(mtm, tag);
    segment.setControlSize(NSControlSize::Small);
    segment
}

pub fn symbol_image(name: &str) -> Option<Retained<NSImage>> {
    NSImage::imageWithSystemSymbolName_accessibilityDescription(&ns(name), None)
}

// ---- Frame placement --------------------------------------------------------

pub fn place(view: &NSView, x: f64, y: f64, w: f64, h: f64) {
    view.setFrame(NSRect::new(NSPoint::new(x, y), NSSize::new(w, h)));
}
