use std::ptr::{self, NonNull};

use objc2_application_services::{
    kAXTrustedCheckOptionPrompt, AXError, AXIsProcessTrusted, AXIsProcessTrustedWithOptions,
    AXUIElement, AXValue, AXValueType,
};
use objc2_core_foundation::{CFBoolean, CFDictionary, CFRange, CFRetained, CFString, CFType};

const TIMEOUT: f32 = 0.08;
const MAX_FIELD_LENGTH: isize = 65536;

pub fn prompt_for_permission() -> bool {
    // This dictionary contains the Accessibility prompt key and its documented Boolean value.
    unsafe {
        let options =
            CFDictionary::from_slices(&[kAXTrustedCheckOptionPrompt], &[CFBoolean::new(true)]);
        AXIsProcessTrustedWithOptions(Some(options.as_opaque()))
    }
}

pub fn is_trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

pub struct LauncherAccessibility {
    focused: CFRetained<CFString>,
    value: CFRetained<CFString>,
    selection: CFRetained<CFString>,
}

impl LauncherAccessibility {
    pub fn new() -> Self {
        // The AX attribute constants are C string macros in AXAttributeConstants.h.
        Self {
            focused: CFString::from_str("AXFocusedUIElement"),
            value: CFString::from_str("AXValue"),
            selection: CFString::from_str("AXSelectedTextRange"),
        }
    }

    pub fn replace(&self, pid: i32, delete: usize, replacement: &[u16]) -> bool {
        self.try_replace(pid, delete, replacement).is_some()
    }

    fn try_replace(&self, pid: i32, delete: usize, replacement: &[u16]) -> Option<()> {
        // Query the event's target, so a focus change cannot redirect an edit to another app.
        let application = unsafe { AXUIElement::new_application(pid) };
        if unsafe { application.set_messaging_timeout(TIMEOUT) } != AXError::Success {
            return None;
        }
        let focused = attribute(&application, &self.focused)?
            .downcast::<AXUIElement>()
            .ok()?;
        if unsafe { focused.set_messaging_timeout(TIMEOUT) } != AXError::Success {
            return None;
        }
        let value = attribute(&focused, &self.value)?
            .downcast::<CFString>()
            .ok()?;
        let range = attribute(&focused, &self.selection)?
            .downcast::<AXValue>()
            .ok()?;
        let mut selection = CFRange {
            location: 0,
            length: 0,
        };
        if !unsafe { range.value(AXValueType::CFRange, NonNull::from(&mut selection).cast()) } {
            return None;
        }

        let length = value.length();
        if !(0..=MAX_FIELD_LENGTH).contains(&length) {
            return None;
        }
        let mut units = vec![0; length as usize];
        // The input and output buffer lengths match the full CFString range in UTF-16 units.
        unsafe {
            value.characters(
                CFRange {
                    location: 0,
                    length,
                },
                units.as_mut_ptr(),
            )
        };
        let caret = replace_units(&mut units, selection, delete, replacement)?;
        let updated =
            unsafe { CFString::with_characters(None, units.as_ptr(), units.len() as isize) }?;
        let caret_value =
            unsafe { AXValue::new(AXValueType::CFRange, NonNull::from(&caret).cast()) }?;
        if unsafe { focused.set_attribute_value(&self.value, updated.as_ref()) } != AXError::Success
        {
            return None;
        }
        // The value is already committed. A failed caret write must not trigger a second edit.
        let _ = unsafe { focused.set_attribute_value(&self.selection, caret_value.as_ref()) };
        Some(())
    }
}

fn attribute(element: &AXUIElement, key: &CFString) -> Option<CFRetained<CFType>> {
    let mut value = ptr::null();
    let status = unsafe { element.copy_attribute_value(key, NonNull::from(&mut value)) };
    // The Copy rule gives us ownership of any returned object.
    let value = NonNull::new(value.cast_mut()).map(|value| unsafe { CFRetained::from_raw(value) });
    if status == AXError::Success {
        value
    } else {
        None
    }
}

fn replace_units(
    value: &mut Vec<u16>,
    selection: CFRange,
    delete: usize,
    replacement: &[u16],
) -> Option<CFRange> {
    let caret = usize::try_from(selection.location).ok()?;
    let end = caret.checked_add(usize::try_from(selection.length).ok()?)?;
    let start = caret.checked_sub(delete)?;
    if end > value.len() {
        return None;
    }
    // Reject stale ranges that would split an existing surrogate pair.
    if [start, caret, end].into_iter().any(|index| {
        index > 0
            && value
                .get(index)
                .is_some_and(|unit| (0xdc00..=0xdfff).contains(unit))
            && (0xd800..=0xdbff).contains(&value[index - 1])
    }) {
        return None;
    }
    value.splice(start..end, replacement.iter().copied());
    Some(CFRange {
        location: (start + replacement.len()) as isize,
        length: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_a_composed_vowel_and_selection_at_a_utf16_caret() {
        let mut value: Vec<_> = "😀o\u{0302}xy!".encode_utf16().collect();
        let caret = replace_units(
            &mut value,
            CFRange {
                location: 4,
                length: 2,
            },
            2,
            &[0x1ed3],
        )
        .unwrap();
        assert_eq!(String::from_utf16(&value).unwrap(), "😀ồ!");
        assert_eq!((caret.location, caret.length), (3, 0));
    }

    #[test]
    fn refuses_a_range_that_splits_a_surrogate_pair() {
        let mut value: Vec<_> = "😀".encode_utf16().collect();
        assert!(replace_units(
            &mut value,
            CFRange {
                location: 2,
                length: 0
            },
            1,
            &[97]
        )
        .is_none());
        assert_eq!(String::from_utf16(&value).unwrap(), "😀");
    }
}
