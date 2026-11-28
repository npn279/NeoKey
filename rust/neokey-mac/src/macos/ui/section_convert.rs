use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{NSPopUpButton, NSSwitch, NSTextField, NSView};

use super::convert_map::{CaseMode, CODE_TABLE_LABELS};
use super::style::*;
use super::tag;
use super::UiSnapshot;
use crate::macos::status_menu::MenuTarget;

pub struct ConvertSection {
    view: Retained<FlippedView>,
    from_popup: Retained<NSPopUpButton>,
    to_popup: Retained<NSPopUpButton>,
    case_popup: Retained<NSPopUpButton>,
    remove_mark: Retained<NSSwitch>,
    input: Retained<NSTextField>,
    output: Retained<NSTextField>,
}

impl ConvertSection {
    pub fn build(mtm: MainThreadMarker, target: &MenuTarget, w: f64, h: f64) -> Self {
        let view = FlippedView::new(
            mtm,
            objc2_foundation::NSRect::new(
                objc2_foundation::NSPoint::new(0.0, 0.0),
                objc2_foundation::NSSize::new(w, h),
            ),
        );
        let title = title_label(mtm, "Chuyển mã", 22.0);
        place(&title, 24.0, 16.0, 200.0, 30.0);
        view.addSubview(&title);

        let from_popup = NSPopUpButton::new(mtm);
        let to_popup = NSPopUpButton::new(mtm);
        for label in CODE_TABLE_LABELS {
            from_popup.addItemWithTitle(&ns(label));
            to_popup.addItemWithTitle(&ns(label));
        }
        place(&from_popup, 24.0, 60.0, 200.0, 26.0);
        place(&to_popup, 240.0, 60.0, 200.0, 26.0);

        let swap = make_button(mtm, "Đổi ↔", tag::CONVERT_SWAP);
        place(&swap, 460.0, 58.0, 72.0, 28.0);

        let case_popup = NSPopUpButton::new(mtm);
        for mode in CaseMode::ALL {
            case_popup.addItemWithTitle(&ns(mode.label()));
        }
        place(&case_popup, 24.0, 100.0, 220.0, 26.0);
        let remove_mark = make_switch(mtm, tag::CONVERT_RUN + 10, false);
        place(&remove_mark, 280.0, 100.0, 44.0, 26.0);
        let rm_lab = label(mtm, "Bỏ dấu", 13.0, false);
        place(&rm_lab, 330.0, 102.0, 100.0, 18.0);

        let input = NSTextField::new(mtm);
        input.setEditable(true);
        place(&input, 24.0, 150.0, w - 48.0, 24.0);
        let output = NSTextField::new(mtm);
        output.setEditable(false);
        place(&output, 24.0, 190.0, w - 48.0, 24.0);

        let paste = make_button(mtm, "Dán", tag::CONVERT_PASTE);
        let copy = make_button(mtm, "Sao chép", tag::CONVERT_COPY);
        let run = make_button(mtm, "Chuyển", tag::CONVERT_RUN);
        place(&paste, 24.0, 230.0, 80.0, 28.0);
        place(&copy, 110.0, 230.0, 100.0, 28.0);
        place(&run, w - 110.0, 230.0, 86.0, 28.0);

        unsafe {
            from_popup.setTarget(Some(target));
            from_popup.setAction(Some(sel!(uiAction:)));
            to_popup.setTarget(Some(target));
            to_popup.setAction(Some(sel!(uiAction:)));
            case_popup.setTarget(Some(target));
            case_popup.setAction(Some(sel!(uiAction:)));
            remove_mark.setTarget(Some(target));
            remove_mark.setAction(Some(sel!(uiAction:)));
            swap.setTarget(Some(target));
            swap.setAction(Some(sel!(uiAction:)));
            paste.setTarget(Some(target));
            paste.setAction(Some(sel!(uiAction:)));
            copy.setTarget(Some(target));
            copy.setAction(Some(sel!(uiAction:)));
            run.setTarget(Some(target));
            run.setAction(Some(sel!(uiAction:)));
        }

        for v in [
            &from_popup as &NSView,
            &to_popup,
            &case_popup,
            &swap,
            &input,
            &output,
            &paste,
            &copy,
            &run,
        ] {
            view.addSubview(v);
        }
        view.addSubview(&remove_mark);
        view.addSubview(&rm_lab);

        Self {
            view,
            from_popup,
            to_popup,
            case_popup,
            remove_mark,
            input,
            output,
        }
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }

    pub fn input_text(&self) -> String {
        self.input.stringValue().to_string()
    }

    pub fn set_output(&self, text: &str) {
        self.output.setStringValue(&ns(text));
    }

    pub fn ui_options(&self) -> super::convert_map::ConvertUiOptions {
        super::convert_map::ConvertUiOptions {
            from_code: self.from_popup.indexOfSelectedItem() as u8,
            to_code: self.to_popup.indexOfSelectedItem() as u8,
            case_mode: CaseMode::ALL[self.case_popup.indexOfSelectedItem().clamp(0, 4) as usize],
            remove_mark: switch_is_on(&self.remove_mark),
        }
    }

    pub fn set_input(&self, text: &str) {
        self.input.setStringValue(&ns(text));
    }

    pub fn set_from_to(&self, from: u8, to: u8) {
        self.from_popup.selectItemAtIndex(from as isize);
        self.to_popup.selectItemAtIndex(to as isize);
    }

    pub fn refresh(&self, _snap: &UiSnapshot) {}
}
