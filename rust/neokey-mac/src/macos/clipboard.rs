use objc2_app_kit::{NSPasteboard, NSPasteboardItem, NSPasteboardTypeHTML, NSPasteboardTypeString};
use objc2_foundation::{NSArray, NSString};

use super::ui::ConvertUiOptions;

#[derive(Debug, PartialEq, Eq)]
pub enum Result {
    Converted,
    Empty,
    Failed,
}

pub fn quick_convert(options: ConvertUiOptions) -> Result {
    convert_on_board(&NSPasteboard::generalPasteboard(), options)
}

fn convert_on_board(board: &NSPasteboard, options: ConvertUiOptions) -> Result {
    let item = NSPasteboardItem::new();
    let mut has_text = false;
    for flavor in unsafe { [NSPasteboardTypeHTML, NSPasteboardTypeString] } {
        let Some(text) = board.stringForType(flavor) else {
            continue;
        };
        let Ok(converted) = options.to_engine().convert(&text.to_string()) else {
            return Result::Failed;
        };
        if !item.setString_forType(&NSString::from_str(&converted), flavor) {
            return Result::Failed;
        }
        has_text = true;
    }
    if !has_text {
        return Result::Empty;
    }

    board.clearContents();
    let item = objc2::runtime::ProtocolObject::from_retained(item);
    let items = NSArray::from_retained_slice(&[item]);
    if board.writeObjects(&items) {
        Result::Converted
    } else {
        Result::Failed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macos::ui::CaseMode;

    #[test]
    fn quick_convert_preserves_both_clipboard_representations() {
        let board = NSPasteboard::pasteboardWithUniqueName();
        let source = NSPasteboardItem::new();
        let (html, plain) = unsafe { (NSPasteboardTypeHTML, NSPasteboardTypeString) };
        assert!(source.setString_forType(&NSString::from_str("<p>Việt Nam</p>"), html));
        assert!(source.setString_forType(&NSString::from_str("Việt Nam"), plain));
        let items =
            NSArray::from_retained_slice(&[objc2::runtime::ProtocolObject::from_retained(source)]);
        assert!(board.writeObjects(&items));
        let options = ConvertUiOptions {
            from_code: 0,
            to_code: 0,
            case_mode: CaseMode::Lower,
            remove_mark: true,
        };
        assert_eq!(convert_on_board(&board, options), Result::Converted);
        assert_eq!(
            board.stringForType(html).unwrap().to_string(),
            "<p>viet nam</p>"
        );
        assert_eq!(board.stringForType(plain).unwrap().to_string(), "viet nam");
        board.clearContents();
        assert_eq!(convert_on_board(&board, options), Result::Empty);
    }
}
