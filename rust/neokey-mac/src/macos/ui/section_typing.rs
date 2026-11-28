use objc2::rc::Retained;
use objc2::{sel, MainThreadMarker};
use objc2_app_kit::{NSScrollView, NSSwitch, NSView};

use super::style::*;
use super::tag;
use super::UiSnapshot;
use crate::macos::status_menu::MenuTarget;

macro_rules! pref_switch {
    ($mtm:expr, $target:expr, $view:expr, $y:expr, $label:expr, $tag:expr) => {{
        let sw = make_switch($mtm, $tag, false);
        place(&sw, $view.bounds().size.width - 70.0, $y, 44.0, 26.0);
        let lab = label($mtm, $label, 14.0, false);
        place(&lab, 24.0, $y, 420.0, 20.0);
        unsafe {
            sw.setTarget(Some($target));
            sw.setAction(Some(sel!(uiAction:)));
        }
        $view.addSubview(&lab);
        $view.addSubview(&sw);
        sw
    }};
}

pub struct TypingSection {
    view: Retained<FlippedView>,
    switches: Vec<Retained<NSSwitch>>,
}

const PREFS: &[(&str, isize)] = &[
    ("Kiểm tra chính tả", 10),
    ("Chính tả hiện đại", 11),
    ("Telex nhanh", 12),
    ("Khôi phục khi sai chính tả", 13),
    ("Viết hoa chữ cái đầu", 14),
    ("Cho phép phụ âm z + ZWJ", 15),
    ("Quick start consonant", 16),
    ("Quick end consonant", 17),
    ("Dùng macro", 18),
    ("Macro trong chế độ Anh", 19),
    ("Tự viết hoa macro", 20),
    ("Chặn gõ khi bàn phím không phải tiếng Anh", 21),
    ("Tắt tạm NeoKey", 22),
];

impl TypingSection {
    pub fn build(mtm: MainThreadMarker, target: &MenuTarget, w: f64, h: f64) -> Self {
        let scroll = NSScrollView::new(mtm);
        place(&scroll, 0.0, 0.0, w, h);
        let doc = FlippedView::new(
            mtm,
            objc2_foundation::NSRect::new(
                objc2_foundation::NSPoint::new(0.0, 0.0),
                objc2_foundation::NSSize::new(w, 40.0 + PREFS.len() as f64 * 44.0),
            ),
        );
        let title = title_label(mtm, "Kiểu gõ", 22.0);
        place(&title, 24.0, 16.0, 200.0, 30.0);
        doc.addSubview(&title);
        let mut switches = Vec::new();
        for (i, (label, off)) in PREFS.iter().enumerate() {
            let sw = pref_switch!(
                mtm,
                target,
                &doc,
                60.0 + i as f64 * 44.0,
                label,
                tag::SETTING + *off
            );
            switches.push(sw);
        }
        scroll.setDocumentView(Some(&doc));
        let view = FlippedView::new(
            mtm,
            objc2_foundation::NSRect::new(
                objc2_foundation::NSPoint::new(0.0, 0.0),
                objc2_foundation::NSSize::new(w, h),
            ),
        );
        view.addSubview(&scroll);
        Self { view, switches }
    }

    pub fn view(&self) -> &NSView {
        &self.view
    }

    pub fn refresh(&self, snap: &UiSnapshot) {
        let s = &snap.settings;
        let vals = [
            snap.spelling,
            s.vUseModernOrthography != 0,
            s.vQuickTelex != 0,
            s.vRestoreIfWrongSpelling != 0,
            s.vUpperCaseFirstChar != 0,
            s.vAllowConsonantZFWJ != 0,
            s.vQuickStartConsonant != 0,
            s.vQuickEndConsonant != 0,
            s.vUseMacro != 0,
            s.vUseMacroInEnglishMode != 0,
            s.vAutoCapsMacro != 0,
            s.vOtherLanguage != 0,
            s.vTempOffOpenKey != 0,
        ];
        for (sw, on) in self.switches.iter().zip(vals) {
            set_switch(sw, on);
        }
    }
}
