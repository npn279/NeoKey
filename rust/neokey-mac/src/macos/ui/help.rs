//! Small round "?" buttons that explain a setting on click.
//!
//! Tooltips need a hover and a delay, and only show while NeoKey is the active
//! app. A help button shows the same text at once, in a popover next to it.

use std::cell::RefCell;

use objc2::rc::Retained;
use objc2::runtime::NSObject;
use objc2::{define_class, msg_send, sel, MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSButton, NSCellImagePosition, NSColor, NSFontWeightRegular, NSImageSymbolConfiguration,
    NSPopover, NSPopoverBehavior, NSTextField, NSView, NSViewController,
};
use objc2_foundation::{NSPoint, NSRect, NSRectEdge, NSSize};

use super::style::{ns, place, regular, symbol_image};

/// Width the explanation wraps at inside the popover.
const TEXT_WIDTH: f64 = 280.0;
const PADDING: f64 = 12.0;
/// Gap between a label's text and its help button.
const GAP: f64 = 4.0;
/// Point size of the "?" symbol: a little smaller than the label text.
const ICON_POINT_SIZE: f64 = 11.0;
/// Hit area around the symbol, so the small icon is still easy to click.
const BUTTON_SIZE: f64 = 18.0;

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    struct HelpTarget;

    impl HelpTarget {
        #[unsafe(method(showHelp:))]
        fn show_help(&self, sender: &NSButton) {
            // The button keeps its text as its tooltip, so hovering still works.
            if let Some(text) = sender.toolTip() {
                show_popover(self.mtm(), sender, &text.to_string());
            }
        }
    }
);

thread_local! {
    static TARGET: RefCell<Option<Retained<HelpTarget>>> = const { RefCell::new(None) };
    /// The open popover; kept alive here until the next one replaces it.
    static OPEN: RefCell<Option<Retained<NSPopover>>> = const { RefCell::new(None) };
}

fn target(mtm: MainThreadMarker) -> Retained<HelpTarget> {
    TARGET.with(|slot| {
        slot.borrow_mut()
            .get_or_insert_with(|| unsafe { msg_send![HelpTarget::alloc(mtm), init] })
            .clone()
    })
}

/// A standalone help button that explains `text` when clicked.
pub fn help_button(mtm: MainThreadMarker, text: &str) -> Retained<NSButton> {
    // A faint borderless "?" in a circle, so it hints without competing with
    // the setting's own label.
    let button = NSButton::new(mtm);
    button.setBordered(false);
    button.setTitle(&ns(""));
    match symbol_image("questionmark.circle") {
        Some(image) => {
            let config = NSImageSymbolConfiguration::configurationWithPointSize_weight(
                ICON_POINT_SIZE,
                unsafe { NSFontWeightRegular },
            );
            let image = image.imageWithSymbolConfiguration(&config).unwrap_or(image);
            button.setImage(Some(&image));
            button.setImagePosition(NSCellImagePosition::ImageOnly);
        }
        None => button.setTitle(&ns("?")),
    }
    button.setContentTintColor(Some(&NSColor::tertiaryLabelColor()));
    button.setToolTip(Some(&ns(text)));
    let target = target(mtm);
    unsafe {
        button.setTarget(Some(&target));
        button.setAction(Some(sel!(showHelp:)));
    }
    button
}

/// Puts a help button right after the visible text of `label`, in `parent`
/// (the label's superview, laid out in the same coordinates).
pub fn add_help(mtm: MainThreadMarker, parent: &NSView, label: &NSTextField, text: &str) {
    let button = help_button(mtm, text);
    let size = NSSize::new(BUTTON_SIZE, BUTTON_SIZE);
    let frame = label.frame();
    let text_w = label.intrinsicContentSize().width.min(frame.size.width);
    let y = frame.origin.y + (frame.size.height - size.height) / 2.0;
    place(
        &button,
        frame.origin.x + text_w + GAP,
        y,
        size.width,
        size.height,
    );
    parent.addSubview(&button);
}

fn show_popover(mtm: MainThreadMarker, anchor: &NSButton, text: &str) {
    OPEN.with(|slot| {
        if let Some(old) = slot.borrow_mut().take() {
            old.close();
        }
    });
    let field = NSTextField::wrappingLabelWithString(&ns(text), mtm);
    field.setFont(Some(&regular(13.0)));
    field.setPreferredMaxLayoutWidth(TEXT_WIDTH);
    field.setSelectable(false);
    let fit = field.fittingSize();
    let content = NSSize::new(
        fit.width.min(TEXT_WIDTH) + 2.0 * PADDING,
        fit.height + 2.0 * PADDING,
    );
    let root = NSView::initWithFrame(
        NSView::alloc(mtm),
        NSRect::new(NSPoint::new(0.0, 0.0), content),
    );
    field.setFrame(NSRect::new(
        NSPoint::new(PADDING, PADDING),
        NSSize::new(fit.width.min(TEXT_WIDTH), fit.height),
    ));
    root.addSubview(&field);

    let controller = NSViewController::new(mtm);
    controller.setView(&root);
    let popover = NSPopover::new(mtm);
    popover.setBehavior(NSPopoverBehavior::Transient);
    popover.setContentViewController(Some(&controller));
    popover.setContentSize(content);
    popover.showRelativeToRect_ofView_preferredEdge(anchor.bounds(), anchor, NSRectEdge::MaxX);
    OPEN.with(|slot| *slot.borrow_mut() = Some(popover));
}
