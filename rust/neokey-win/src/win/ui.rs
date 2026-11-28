use super::{
    begin_update, change, convert, error, notice, read_app, save_and_refresh, system, tray, wide,
    with_app, STARTUP_WORK,
};
use neokey_engine::macros::Macros;
use neokey_win::{
    data::{checked_macros, macro_fits, RELEASES_URL},
    settings::{
        Hotkey, Preferences, Setting, ALT, CODE_TABLES, CTRL, INPUT_TYPES, MACRO_DATA, SHIFT, WIN,
    },
};
use std::{
    cell::Cell,
    ffi::OsString,
    io,
    mem::{size_of, zeroed},
    os::windows::ffi::OsStringExt,
    path::PathBuf,
    ptr::{null, null_mut},
};
use windows_sys::{
    w,
    Win32::{
        Foundation::*,
        Graphics::Gdi::{GetStockObject, COLOR_BTNFACE, DEFAULT_GUI_FONT},
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::{Dialogs::*, EM_SETLIMITTEXT},
            Input::KeyboardAndMouse::*,
            Shell::IsUserAnAdmin,
            WindowsAndMessaging::*,
        },
    },
};

const PANEL_CLASS: *const u16 = w!("NeoKeyRustPanel");
const PREF_ID: u16 = 100;
const CTRL_BOX: u16 = 300;
const KEY_BOX: u16 = 304;
const BEEP_BOX: u16 = 305;
const RESET: u16 = 306;
const SHOW_MACROS: u16 = 307;
const CHECK_UPDATE: u16 = 308;
const SOURCE: u16 = 309;
const RELEASES: u16 = 310;
const SWAP: u16 = 311;
const CONVERT: u16 = 312;
const MACRO_LIST: u16 = 320;
const MACRO_NAME: u16 = 321;
const MACRO_CONTENT: u16 = 322;
const MACRO_SAVE: u16 = 323;
const MACRO_DELETE: u16 = 324;
const MACRO_IMPORT: u16 = 325;
const MACRO_EXPORT: u16 = 326;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum Panel {
    Main,
    Macros,
    Convert,
    About,
}

impl Panel {
    const ALL: [Self; 4] = [Self::Main, Self::Macros, Self::Convert, Self::About];

    fn title_size(self) -> (&'static str, i32, i32) {
        match self {
            Self::Main => ("NeoKey – Bảng điều khiển", 753, 596),
            Self::Macros => ("NeoKey – Bảng gõ tắt", 665, 414),
            Self::Convert => ("NeoKey – Công cụ chuyển mã", 570, 329),
            Self::About => ("Giới thiệu NeoKey", 515, 222),
        }
    }
}

thread_local! {
    static WINDOWS: Cell<[HWND; 4]> = const { Cell::new([null_mut(); 4]) };
}

fn window(panel: Panel) -> HWND {
    WINDOWS.get()[panel as usize]
}

fn store_window(panel: Panel, window: HWND) {
    let mut windows = WINDOWS.get();
    windows[panel as usize] = window;
    WINDOWS.set(windows);
}

pub unsafe fn register(module: HINSTANCE, icon: HICON) -> io::Result<()> {
    let mut class: WNDCLASSW = zeroed();
    class.lpfnWndProc = Some(proc);
    class.hInstance = module;
    class.hIcon = icon;
    class.hCursor = LoadCursorW(null_mut(), IDC_ARROW);
    class.hbrBackground = (COLOR_BTNFACE + 1) as usize as _;
    class.lpszClassName = PANEL_CLASS;
    if RegisterClassW(&class) == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub unsafe fn show(owner: HWND, panel: Panel) {
    let mut handle = window(panel);
    if handle.is_null() {
        let (title, width, height) = panel.title_size();
        let style = WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
        let mut outer = RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };
        AdjustWindowRectEx(
            &mut outer,
            style,
            0,
            WS_EX_CONTROLPARENT | WS_EX_DLGMODALFRAME,
        );
        handle = CreateWindowExW(
            WS_EX_CONTROLPARENT | WS_EX_DLGMODALFRAME,
            PANEL_CLASS,
            wide(title).as_ptr(),
            style,
            (GetSystemMetrics(SM_CXSCREEN) - outer.right + outer.left).max(0) / 2,
            (GetSystemMetrics(SM_CYSCREEN) - outer.bottom + outer.top).max(0) / 2,
            outer.right - outer.left,
            outer.bottom - outer.top,
            owner,
            null_mut(),
            GetModuleHandleW(null()),
            null(),
        );
        if handle.is_null() {
            error(owner, "Không mở được cửa sổ NeoKey.");
            return;
        }
        store_window(panel, handle);
        match panel {
            Panel::Main => init_main(handle),
            Panel::Macros => init_macros(handle),
            Panel::Convert => init_convert(handle),
            Panel::About => init_about(handle),
        }
        refresh();
    }
    ShowWindow(handle, SW_RESTORE);
    SetForegroundWindow(handle);
    let first = GetNextDlgTabItem(handle, null_mut(), 0);
    if !first.is_null() {
        SetFocus(first);
    }
}

pub unsafe fn handle_message(message: &MSG) -> bool {
    for panel in Panel::ALL {
        let dialog = window(panel);
        if dialog.is_null() || IsChild(dialog, message.hwnd) == 0 && message.hwnd != dialog {
            continue;
        }
        if message.hwnd == GetDlgItem(dialog, KEY_BOX.into())
            && matches!(message.message, WM_KEYDOWN | WM_SYSKEYDOWN)
        {
            let vk = message.wParam as u16;
            if vk != VK_TAB && vk != VK_ESCAPE {
                if let Some(key) = neokey_win::typing::capture_key(vk) {
                    capture(dialog, panel, key);
                }
                return true;
            }
        }
        return IsDialogMessageW(dialog, message) != 0;
    }
    false
}

pub unsafe fn is_capture_focused(front: HWND) -> bool {
    let focus = GetFocus();
    !focus.is_null()
        && [Panel::Main, Panel::Convert].iter().any(|&panel| {
            let dialog = window(panel);
            dialog == front && !dialog.is_null() && GetDlgItem(dialog, KEY_BOX.into()) == focus
        })
}

pub unsafe fn close_all() {
    for panel in Panel::ALL {
        let dialog = window(panel);
        if !dialog.is_null() {
            DestroyWindow(dialog);
        }
    }
}

fn pref_id(setting: Setting) -> u16 {
    PREF_ID + setting as u16
}

unsafe fn control(
    dialog: HWND,
    class: *const u16,
    label: &str,
    id: u16,
    styles: u32,
    rect: [i32; 4],
) -> HWND {
    let [x, y, width, height] = rect;
    let handle = CreateWindowExW(
        0,
        class,
        wide(label).as_ptr(),
        WS_CHILD | WS_VISIBLE | styles,
        x,
        y,
        width,
        height,
        dialog,
        usize::from(id) as HMENU,
        GetModuleHandleW(null()),
        null(),
    );
    if !handle.is_null() {
        SendMessageW(
            handle,
            WM_SETFONT,
            GetStockObject(DEFAULT_GUI_FONT) as usize,
            1,
        );
    }
    handle
}

unsafe fn label(dialog: HWND, text: &str, rect: [i32; 4]) {
    control(dialog, w!("STATIC"), text, 0, 0, rect);
}

unsafe fn button(dialog: HWND, text: &str, id: u16, rect: [i32; 4]) {
    control(dialog, w!("BUTTON"), text, id, WS_TABSTOP, rect);
}

unsafe fn checkbox(dialog: HWND, text: &str, id: u16, rect: [i32; 4]) {
    control(
        dialog,
        w!("BUTTON"),
        text,
        id,
        WS_TABSTOP | BS_AUTOCHECKBOX as u32,
        rect,
    );
}

unsafe fn group(dialog: HWND, text: &str, rect: [i32; 4]) {
    control(dialog, w!("BUTTON"), text, 0, BS_GROUPBOX as u32, rect);
}

unsafe fn drop_down(dialog: HWND, setting: Setting, items: &[&str], rect: [i32; 4]) {
    let combo = control(
        dialog,
        w!("COMBOBOX"),
        "",
        pref_id(setting),
        WS_TABSTOP | WS_VSCROLL | CBS_DROPDOWNLIST as u32,
        rect,
    );
    for item in items {
        SendMessageW(combo, CB_ADDSTRING, 0, wide(item).as_ptr() as isize);
    }
}

unsafe fn shortcut_controls(dialog: HWND, x: i32, y: i32, beep: bool) {
    for (i, (name, width)) in [("Ctrl", 55), ("Alt", 55), ("Win", 55), ("Shift", 62)]
        .into_iter()
        .enumerate()
    {
        checkbox(
            dialog,
            name,
            CTRL_BOX + i as u16,
            [x + i as i32 * 58, y, width, 23],
        );
    }
    control(
        dialog,
        w!("EDIT"),
        "",
        KEY_BOX,
        WS_BORDER | WS_TABSTOP | ES_READONLY as u32,
        [x + 237, y, 90, 23],
    );
    if beep {
        checkbox(dialog, "Kêu Beep", BEEP_BOX, [x + 345, y, 100, 23]);
    }
    label(dialog, "Backspace để xóa phím", [x + 237, y + 26, 190, 18]);
}

const GENERAL: &[(Setting, &str)] = &[
    (
        Setting::ModernOrthography,
        "Đặt dấu oà, uý (thay vì òa, úy)",
    ),
    (Setting::BrowserFix, "Sửa lỗi gợi ý (trình duyệt, Excel)"),
    (Setting::Spelling, "Kiểm tra chính tả"),
    (Setting::RestoreSpelling, "Tự khôi phục phím với từ sai"),
    (
        Setting::ForeignConsonants,
        "Cho phép z, w, j, f làm phụ âm đầu",
    ),
    (
        Setting::TemporarySpelling,
        "Tạm tắt chính tả bằng phím Ctrl",
    ),
    (Setting::SmartSwitch, "Loại trừ ứng dụng thông minh"),
    (Setting::FirstLetterCaps, "Viết hoa chữ cái đầu câu"),
    (Setting::RememberTable, "Ghi nhớ bảng mã theo ứng dụng"),
    (
        Setting::OtherLanguages,
        "Cho phép gõ trong các ngôn ngữ khác",
    ),
    (Setting::TemporaryEngine, "Tạm tắt NeoKey bằng phím Alt"),
];

const MACRO_OPTIONS: &[(Setting, &str)] = &[
    (Setting::UseMacro, "Cho phép gõ tắt"),
    (Setting::EnglishMacro, "Gõ tắt cả khi tắt gõ tiếng Việt"),
    (Setting::MacroCaps, "Tự động viết hoa theo phím tắt"),
    (Setting::QuickTelex, "Gõ nhanh (cc=ch, gg=gi, nn=ng,...)"),
    (Setting::QuickStart, "Gõ tắt phụ âm đầu: f→ph, j→gi, w→qu"),
    (Setting::QuickEnd, "Gõ tắt phụ âm cuối: g→ng, h→nh, k→ch"),
];

const SYSTEM_OPTIONS: &[(Setting, &str)] = &[
    (Setting::Metro, "Hỗ trợ ứng dụng Metro"),
    (Setting::DesktopShortcut, "Tạo biểu tượng trên Desktop"),
    (Setting::ShowAtStartup, "Bật bảng này khi khởi động"),
    (Setting::GrayIcon, "Biểu tượng hiện đại"),
    (Setting::RunWithWindows, "Khởi động cùng Windows"),
    (Setting::ChromiumFix, "Sửa lỗi trên Chromium (beta)"),
    (Setting::Administrator, "Chạy với quyền Admin"),
    (Setting::StepByStep, "Dùng Clipboard để gửi phím"),
    (Setting::CheckUpdate, "Kiểm tra bản mới lúc khởi động"),
];

unsafe fn options(dialog: HWND, entries: &[(Setting, &str)], x: i32, y: i32) {
    for (index, &(setting, title)) in entries.iter().enumerate() {
        checkbox(
            dialog,
            title,
            pref_id(setting),
            [x, y + index as i32 * 22, 335, 22],
        );
    }
}

unsafe fn init_main(dialog: HWND) {
    group(dialog, "Điều khiển", [12, 9, 727, 126]);
    label(dialog, "Chế độ gõ:", [25, 32, 74, 20]);
    drop_down(
        dialog,
        Setting::Language,
        &["Tiếng Anh", "Tiếng Việt"],
        [102, 27, 135, 100],
    );
    label(dialog, "Kiểu gõ:", [256, 32, 54, 20]);
    drop_down(dialog, Setting::InputType, INPUT_TYPES, [318, 27, 128, 115]);
    label(dialog, "Bảng mã:", [460, 32, 65, 20]);
    drop_down(dialog, Setting::CodeTable, CODE_TABLES, [535, 27, 187, 122]);
    label(dialog, "Phím chuyển:", [25, 71, 96, 24]);
    shortcut_controls(dialog, 126, 67, true);

    group(dialog, "Tùy chọn", [12, 145, 350, 271]);
    options(dialog, GENERAL, 24, 165);
    group(dialog, "Gõ tắt và gõ nhanh", [371, 145, 368, 190]);
    options(dialog, MACRO_OPTIONS, 384, 165);
    button(dialog, "Bảng gõ tắt...", SHOW_MACROS, [384, 300, 137, 25]);
    group(dialog, "Hệ thống", [12, 424, 727, 122]);
    options(dialog, &SYSTEM_OPTIONS[..4], 24, 446);
    // Fit the remaining system controls in two columns, including the fifth option.
    options(dialog, &SYSTEM_OPTIONS[5..], 384, 446);
    checkbox(
        dialog,
        SYSTEM_OPTIONS[4].1,
        pref_id(Setting::RunWithWindows),
        [384, 358, 330, 22],
    );
    button(
        dialog,
        "Kiểm tra bản mới...",
        CHECK_UPDATE,
        [384, 386, 162, 25],
    );
    button(dialog, "Giới thiệu...", tray::ABOUT, [557, 386, 120, 25]);
    button(dialog, "Tắt NeoKey", tray::EXIT, [20, 557, 129, 28]);
    button(dialog, "Cài đặt gốc", RESET, [440, 557, 130, 28]);
    button(dialog, "Đóng", IDCANCEL as u16, [590, 557, 135, 28]);
}

const CASE_OPTIONS: &[Setting] = &[
    Setting::AllCaps,
    Setting::AllLower,
    Setting::SentenceCaps,
    Setting::WordCaps,
];
const CONVERT_OPTIONS: &[(Setting, &str)] = &[
    (Setting::AllCaps, "Chuyển sang chữ HOA"),
    (Setting::AllLower, "Chuyển sang chữ thường"),
    (Setting::RemoveMarks, "Loại bỏ dấu tiếng Việt"),
    (Setting::SentenceCaps, "Đặt chữ hoa đầu câu"),
    (Setting::WordCaps, "Đặt chữ hoa sau mỗi từ"),
    (Setting::QuietConvert, "Thông báo khi hoàn thành"),
];

unsafe fn init_convert(dialog: HWND) {
    group(dialog, "Tùy chọn", [12, 9, 544, 103]);
    options(dialog, &CONVERT_OPTIONS[..3], 25, 30);
    options(dialog, &CONVERT_OPTIONS[3..], 277, 30);
    group(dialog, "Chuyển mã Clipboard", [12, 119, 544, 135]);
    label(dialog, "Bảng mã nguồn", [26, 140, 170, 20]);
    label(dialog, "Bảng mã đích", [330, 140, 205, 20]);
    drop_down(dialog, Setting::FromTable, CODE_TABLES, [26, 162, 205, 125]);
    button(dialog, "Đảo", SWAP, [247, 162, 64, 24]);
    drop_down(dialog, Setting::ToTable, CODE_TABLES, [330, 162, 205, 125]);
    label(dialog, "Chuyển mã nhanh:", [26, 203, 131, 20]);
    shortcut_controls(dialog, 167, 200, false);
    button(dialog, "Chuyển mã", CONVERT, [296, 277, 130, 30]);
    button(dialog, "Đóng", IDCANCEL as u16, [443, 277, 102, 30]);
}

unsafe fn init_macros(dialog: HWND) {
    label(dialog, "Từ gõ tắt", [15, 16, 160, 20]);
    label(dialog, "Nội dung thay thế", [185, 16, 320, 20]);
    let name = control(
        dialog,
        w!("EDIT"),
        "",
        MACRO_NAME,
        WS_TABSTOP | WS_BORDER | ES_AUTOHSCROLL as u32,
        [15, 38, 156, 25],
    );
    let content = control(
        dialog,
        w!("EDIT"),
        "",
        MACRO_CONTENT,
        WS_TABSTOP | WS_BORDER | WS_VSCROLL | (ES_MULTILINE | ES_AUTOVSCROLL) as u32,
        [185, 38, 461, 62],
    );
    SendMessageW(name, EM_SETLIMITTEXT, 255, 0);
    SendMessageW(content, EM_SETLIMITTEXT, 65535, 0);
    button(dialog, "Thêm / Cập nhật", MACRO_SAVE, [15, 72, 103, 27]);
    button(dialog, "Xóa", MACRO_DELETE, [124, 72, 47, 27]);
    control(
        dialog,
        w!("LISTBOX"),
        "",
        MACRO_LIST,
        WS_TABSTOP
            | WS_BORDER
            | WS_VSCROLL
            | WS_HSCROLL
            | LBS_NOTIFY as u32
            | LBS_NOINTEGRALHEIGHT as u32,
        [15, 111, 631, 243],
    );
    button(dialog, "Nạp từ tệp...", MACRO_IMPORT, [15, 370, 108, 28]);
    button(dialog, "Xuất ra tệp...", MACRO_EXPORT, [133, 370, 111, 28]);
    checkbox(
        dialog,
        "Tự động viết hoa theo phím tắt",
        pref_id(Setting::MacroCaps),
        [257, 371, 264, 25],
    );
    button(dialog, "Đóng", IDCANCEL as u16, [541, 370, 105, 28]);
    reload_macros(dialog);
}

unsafe fn init_about(dialog: HWND) {
    label(
        dialog,
        "NeoKey – Bộ gõ tiếng Việt cho Windows",
        [23, 22, 458, 24],
    );
    label(
        dialog,
        &format!("Phiên bản {}", env!("CARGO_PKG_VERSION")),
        [23, 56, 458, 24],
    );
    label(
        dialog,
        "Phát triển từ OpenKey của Mai Vũ Tuyển. Giấy phép GPL-3.0.",
        [23, 88, 470, 24],
    );
    button(dialog, "Mã nguồn...", SOURCE, [23, 127, 126, 26]);
    button(dialog, "Bản phát hành...", RELEASES, [176, 127, 138, 26]);
    button(
        dialog,
        "Kiểm tra bản mới...",
        CHECK_UPDATE,
        [336, 127, 153, 26],
    );
    button(dialog, "Đóng", IDCANCEL as u16, [384, 173, 105, 26]);
}

fn shortcut_setting(panel: Panel) -> Setting {
    if panel == Panel::Main {
        Setting::SwitchKey
    } else {
        Setting::ConvertKey
    }
}

unsafe fn capture(dialog: HWND, panel: Panel, vk: u8) {
    let setting = shortcut_setting(panel);
    let Some(hotkey) = read_app(|app| Hotkey(app.typing.preferences.get(setting))) else {
        return;
    };
    change(GetWindow(dialog, GW_OWNER), setting, hotkey.with_key(vk).0);
}

unsafe fn set_text(dialog: HWND, id: u16, text: &str) {
    SetDlgItemTextW(dialog, i32::from(id), wide(text).as_ptr());
}

unsafe fn control_text(dialog: HWND, id: u16) -> String {
    let handle = GetDlgItem(dialog, i32::from(id));
    let length = GetWindowTextLengthW(handle);
    let mut buffer = vec![0; length.max(0) as usize + 1];
    let copied = GetWindowTextW(handle, buffer.as_mut_ptr(), buffer.len() as i32);
    String::from_utf16_lossy(&buffer[..copied.max(0) as usize])
}

fn inverted(setting: Setting) -> bool {
    matches!(setting, Setting::StepByStep | Setting::QuietConvert)
}

unsafe fn set_check(dialog: HWND, id: u16, value: bool) {
    SendDlgItemMessageW(dialog, i32::from(id), BM_SETCHECK, usize::from(value), 0);
}

unsafe fn checked(dialog: HWND, id: u16) -> bool {
    SendDlgItemMessageW(dialog, i32::from(id), BM_GETCHECK, 0, 0) == 1
}

unsafe fn fill_shortcut(dialog: HWND, preferences: &Preferences, panel: Panel) {
    let hotkey = Hotkey(preferences.get(shortcut_setting(panel)));
    for (index, bit) in [CTRL, ALT, WIN, SHIFT].into_iter().enumerate() {
        set_check(dialog, CTRL_BOX + index as u16, hotkey.mods() & bit != 0);
    }
    set_text(dialog, KEY_BOX, &tray::key_name(hotkey.vk()));
    if panel == Panel::Main {
        set_check(dialog, BEEP_BOX, hotkey.beep());
    }
}

pub unsafe fn refresh() {
    let Some(preferences) = read_app(|app| app.typing.preferences.clone()) else {
        return;
    };
    for panel in [Panel::Main, Panel::Convert, Panel::Macros] {
        let dialog = window(panel);
        if dialog.is_null() {
            continue;
        }
        if panel != Panel::Macros {
            fill_shortcut(dialog, &preferences, panel);
        }
        for &setting in Setting::ALL {
            let id = i32::from(pref_id(setting));
            if GetDlgItem(dialog, id).is_null() {
                continue;
            }
            if matches!(
                setting,
                Setting::Language
                    | Setting::InputType
                    | Setting::CodeTable
                    | Setting::FromTable
                    | Setting::ToTable
            ) {
                SendDlgItemMessageW(
                    dialog,
                    id,
                    CB_SETCURSEL,
                    preferences.get(setting) as usize,
                    0,
                );
            } else {
                set_check(
                    dialog,
                    id as u16,
                    preferences.enabled(setting) ^ inverted(setting),
                );
            }
        }
        if panel == Panel::Main {
            for setting in [
                Setting::RestoreSpelling,
                Setting::ForeignConsonants,
                Setting::TemporarySpelling,
                Setting::ChromiumFix,
            ] {
                let dependency = if setting == Setting::ChromiumFix {
                    Setting::BrowserFix
                } else {
                    Setting::Spelling
                };
                EnableWindow(
                    GetDlgItem(dialog, i32::from(pref_id(setting))),
                    preferences.enabled(dependency).into(),
                );
            }
        }
    }
}

unsafe fn reload_macros(dialog: HWND) {
    let Some(rows) = read_app(|app| {
        app.typing
            .engine
            .macros()
            .all()
            .map(|(_, entry)| {
                let preview: String = entry
                    .content
                    .lines()
                    .next()
                    .unwrap_or("")
                    .chars()
                    .take(90)
                    .collect();
                format!("{}\t{}", entry.text, preview)
            })
            .collect::<Vec<_>>()
    }) else {
        return;
    };
    let list = GetDlgItem(dialog, i32::from(MACRO_LIST));
    SendMessageW(list, LB_RESETCONTENT, 0, 0);
    for row in rows {
        SendMessageW(list, LB_ADDSTRING, 0, wide(&row).as_ptr() as isize);
    }
}

unsafe fn select_macro(dialog: HWND) {
    let index = SendDlgItemMessageW(dialog, i32::from(MACRO_LIST), LB_GETCURSEL, 0, 0);
    if index >= 0 {
        if let Some(Some((name, content))) = read_app(|app| {
            app.typing
                .engine
                .macros()
                .all()
                .nth(index as usize)
                .map(|(_, entry)| (entry.text.clone(), entry.content.clone()))
        }) {
            set_text(dialog, MACRO_NAME, &name);
            set_text(dialog, MACRO_CONTENT, &content);
        }
    }
}

unsafe fn save_macros(dialog: HWND, macros: Macros) {
    let bytes = macros.save_data();
    let result = with_app(|app| {
        app.registry.write_binary(MACRO_DATA, &bytes)?;
        app.typing
            .engine
            .init_macros(&bytes)
            .map_err(io::Error::other)?;
        app.typing.reset();
        Ok::<_, io::Error>(())
    });
    if let Some(Err(err)) = result {
        error(dialog, &format!("Không lưu được bảng gõ tắt.\n{err}"));
        return;
    }
    reload_macros(dialog);
    set_text(dialog, MACRO_NAME, "");
    set_text(dialog, MACRO_CONTENT, "");
    SetFocus(GetDlgItem(dialog, i32::from(MACRO_NAME)));
}

unsafe fn macro_file(dialog: HWND, save: bool) -> Option<PathBuf> {
    let filter = wide("Tệp văn bản (*.txt)\0*.txt\0Tất cả các tệp (*.*)\0*.*\0");
    let mut path = vec![0; 32768];
    if save {
        let initial = wide("OpenKeyMacro.txt");
        path[..initial.len()].copy_from_slice(&initial);
    }
    let mut dialog_info: OPENFILENAMEW = zeroed();
    dialog_info.lStructSize = size_of::<OPENFILENAMEW>() as u32;
    dialog_info.hwndOwner = dialog;
    dialog_info.lpstrFile = path.as_mut_ptr();
    dialog_info.nMaxFile = path.len() as u32;
    dialog_info.lpstrFilter = filter.as_ptr();
    dialog_info.nFilterIndex = 1;
    dialog_info.lpstrDefExt = w!("txt");
    dialog_info.Flags = OFN_PATHMUSTEXIST
        | OFN_NOCHANGEDIR
        | if save {
            OFN_OVERWRITEPROMPT
        } else {
            OFN_FILEMUSTEXIST
        };
    let success = if save {
        GetSaveFileNameW(&mut dialog_info)
    } else {
        GetOpenFileNameW(&mut dialog_info)
    };
    if success == 0 {
        return None;
    }
    let length = path
        .iter()
        .position(|&code| code == 0)
        .unwrap_or(path.len());
    Some(PathBuf::from(OsString::from_wide(&path[..length])))
}

unsafe fn macro_command(dialog: HWND, id: u16) {
    match id {
        MACRO_SAVE | MACRO_DELETE => {
            let name = control_text(dialog, MACRO_NAME);
            let content = control_text(dialog, MACRO_CONTENT);
            if id == MACRO_SAVE && !macro_fits(&name, &content) {
                error(dialog, "Nhập từ gõ tắt và nội dung. Từ gõ tắt tối đa 255 byte, nội dung tối đa 65.535 byte UTF-8.");
                return;
            }
            let Some((mut macros, table)) = read_app(|app| {
                (
                    app.typing.engine.macros().clone(),
                    app.typing.preferences.get(Setting::CodeTable) as usize,
                )
            }) else {
                return;
            };
            if id == MACRO_SAVE {
                if macros.all().count() >= u16::MAX as usize && !macros.has(&name, table) {
                    error(dialog, "Bảng gõ tắt đã đầy.");
                    return;
                }
                macros.add(&name, &content, table);
            } else if !macros.delete(&name, table) {
                error(dialog, "Bạn hãy chọn từ cần xóa.");
                return;
            }
            save_macros(dialog, macros);
        }
        MACRO_IMPORT => {
            let Some(path) = macro_file(dialog, false) else {
                return;
            };
            let append = match notice(
                dialog,
                "Bạn có muốn giữ lại dữ liệu hiện tại không?",
                MB_YESNOCANCEL | MB_ICONQUESTION,
            ) {
                IDYES => true,
                IDNO => false,
                _ => return,
            };
            match read_app(|app| checked_macros(&app.typing.engine, &path, append)) {
                Some(Ok(macros)) => save_macros(dialog, macros),
                Some(Err(err)) => error(dialog, &format!("Không nạp được tệp gõ tắt.\n{err}")),
                None => {}
            }
        }
        MACRO_EXPORT => {
            let Some(path) = macro_file(dialog, true) else {
                return;
            };
            let snapshot = read_app(|app| app.typing.engine.macros().clone());
            if let Some(Err(err)) = snapshot.map(|macros| macros.save_to_file(path)) {
                error(dialog, &format!("Không xuất được bảng gõ tắt.\n{err}"));
            }
        }
        _ => {}
    }
}

unsafe fn apply_option(owner: HWND, dialog: HWND, setting: Setting, id: u16, notification: u32) {
    if matches!(
        setting,
        Setting::Language
            | Setting::InputType
            | Setting::CodeTable
            | Setting::FromTable
            | Setting::ToTable
    ) {
        if notification == CBN_SELCHANGE {
            let value = SendDlgItemMessageW(dialog, i32::from(id), CB_GETCURSEL, 0, 0);
            if value >= 0 {
                if setting == Setting::Language {
                    tray_command_language(owner, value as u32);
                } else {
                    change(owner, setting, value as u32);
                }
            }
        }
        return;
    }
    if notification != BN_CLICKED {
        return;
    }
    let value = checked(dialog, id) ^ inverted(setting);
    with_app(|app| {
        if CASE_OPTIONS.contains(&setting) {
            for &other in CASE_OPTIONS {
                if other != setting {
                    app.change(other, 0);
                }
            }
        }
        app.change(setting, value.into());
    });
    save_and_refresh(owner);
    match setting {
        Setting::Administrator if value && IsUserAnAdmin() == 0 => {
            if notice(
                dialog,
                "Khởi động lại NeoKey với quyền quản trị viên?",
                MB_YESNO | MB_ICONQUESTION,
            ) == IDYES
            {
                match system::relaunch_elevated(dialog, true) {
                    Ok(()) => PostQuitMessage(0),
                    Err(err) => error(dialog, &format!("Không khởi động lại được NeoKey.\n{err}")),
                }
            }
        }
        Setting::Administrator | Setting::RunWithWindows => {
            PostMessageW(owner, STARTUP_WORK, 0, 0);
        }
        Setting::DesktopShortcut if value => {
            if let Err(err) = system::create_shortcut() {
                error(
                    dialog,
                    &format!("Không tạo được biểu tượng Desktop.\n{err}"),
                );
            }
        }
        _ => {}
    }
}

unsafe fn tray_command_language(owner: HWND, value: u32) {
    if read_app(|app| app.typing.preferences.get(Setting::Language) != value).unwrap_or(false) {
        super::tray_command(owner, tray::TOGGLE);
    }
}

unsafe fn command(dialog: HWND, panel: Panel, id: u16, notification: u32) {
    let owner = GetWindow(dialog, GW_OWNER);
    if let Some(&setting) = id
        .checked_sub(PREF_ID)
        .and_then(|index| Setting::ALL.get(index as usize))
    {
        apply_option(owner, dialog, setting, id, notification);
    } else if id == MACRO_LIST && notification == LBN_SELCHANGE {
        select_macro(dialog);
    } else if notification == BN_CLICKED {
        match id {
            CTRL_BOX..=303 | BEEP_BOX => {
                let setting = shortcut_setting(panel);
                let mods = [CTRL, ALT, WIN, SHIFT]
                    .into_iter()
                    .enumerate()
                    .filter(|(i, _)| checked(dialog, CTRL_BOX + *i as u16))
                    .fold(0, |mods, (_, bit)| mods | bit);
                if let Some(hotkey) = read_app(|app| Hotkey(app.typing.preferences.get(setting))) {
                    change(
                        owner,
                        setting,
                        hotkey
                            .with_mods(mods)
                            .with_beep(if panel == Panel::Main {
                                checked(dialog, BEEP_BOX)
                            } else {
                                hotkey.beep()
                            })
                            .0,
                    );
                }
            }
            RESET => {
                if notice(
                    dialog,
                    "Bạn có chắc chắn muốn thiết lập lại cài đặt gốc?",
                    MB_YESNO | MB_ICONEXCLAMATION,
                ) != IDYES
                {
                    return;
                }
                with_app(|app| {
                    app.typing.preferences.reset_main();
                    app.typing.refresh_settings();
                });
                save_and_refresh(owner);
                PostMessageW(owner, STARTUP_WORK, 0, 0);
            }
            SHOW_MACROS => show(owner, Panel::Macros),
            tray::ABOUT => show(owner, Panel::About),
            CHECK_UPDATE => begin_update(owner, true),
            SOURCE => {
                system::shell_open(dialog, &wide("https://github.com/npn279/NeoKey"));
            }
            RELEASES => {
                system::shell_open(dialog, &wide(RELEASES_URL));
            }
            CONVERT => convert(owner, dialog),
            SWAP => {
                with_app(|app| {
                    let from = app.typing.preferences.get(Setting::FromTable);
                    let to = app.typing.preferences.get(Setting::ToTable);
                    app.change(Setting::FromTable, to);
                    app.change(Setting::ToTable, from);
                });
                save_and_refresh(owner);
            }
            MACRO_SAVE..=MACRO_EXPORT => macro_command(dialog, id),
            tray::EXIT => PostQuitMessage(0),
            id if id == IDCANCEL as u16 => {
                DestroyWindow(dialog);
            }
            _ => {}
        }
    }
}

unsafe extern "system" fn proc(
    dialog: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_COMMAND => {
            if let Some(panel) = Panel::ALL
                .into_iter()
                .find(|&panel| window(panel) == dialog)
            {
                command(dialog, panel, wparam as u16, (wparam >> 16) as u16 as u32);
            }
            0
        }
        WM_CLOSE => {
            DestroyWindow(dialog);
            0
        }
        WM_NCDESTROY => {
            if let Some(panel) = Panel::ALL
                .into_iter()
                .find(|&panel| window(panel) == dialog)
            {
                store_window(panel, null_mut());
            }
            DefWindowProcW(dialog, message, wparam, lparam)
        }
        _ => DefWindowProcW(dialog, message, wparam, lparam),
    }
}
