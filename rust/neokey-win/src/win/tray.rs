use super::{read_app, wide, WM_TRAY};
use neokey_win::settings::{Hotkey, Setting, ALT, CODE_TABLES, CTRL, INPUT_TYPES, SHIFT, WIN};
use std::{
    mem::{size_of, zeroed},
    ptr::null,
};
use windows_sys::Win32::{
    Foundation::{HWND, POINT},
    UI::{Shell::*, WindowsAndMessaging::*},
};

pub const TOGGLE: u16 = 900;
pub const SPELLING: u16 = 901;
pub const SMART_SWITCH: u16 = 902;
pub const USE_MACRO: u16 = 903;
pub const INPUT_TYPE: u16 = 910;
pub const CODE_TABLE: u16 = 930;
pub const CONVERT_TOOL: u16 = 980;
pub const QUICK_CONVERT: u16 = 981;
pub const MACROS: u16 = 990;
pub const CONTROL_PANEL: u16 = 1000;
pub const ABOUT: u16 = 1010;
pub const EXIT: u16 = 2000;

pub struct Icons([HICON; 5]);

impl Icons {
    #[cfg(target_env = "msvc")]
    pub unsafe fn load() -> Self {
        let module = windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(null());
        Self(std::array::from_fn(|index| {
            let width = GetSystemMetrics(if index == 0 { SM_CXICON } else { SM_CXSMICON });
            let height = GetSystemMetrics(if index == 0 { SM_CYICON } else { SM_CYSMICON });
            LoadImageW(
                module,
                (101 + index) as *const u16,
                IMAGE_ICON,
                width,
                height,
                LR_DEFAULTCOLOR,
            )
        }))
    }

    #[cfg(not(target_env = "msvc"))]
    pub unsafe fn load() -> Self {
        Self([
            load_icon(
                include_bytes!(concat!(env!("OUT_DIR"), "/icon.ico")),
                SM_CXICON,
                SM_CYICON,
            ),
            load_icon(
                include_bytes!(concat!(env!("OUT_DIR"), "/StatusViet.ico")),
                SM_CXSMICON,
                SM_CYSMICON,
            ),
            load_icon(
                include_bytes!(concat!(env!("OUT_DIR"), "/StatusEng.ico")),
                SM_CXSMICON,
                SM_CYSMICON,
            ),
            load_icon(
                include_bytes!(concat!(env!("OUT_DIR"), "/StatusViet10.ico")),
                SM_CXSMICON,
                SM_CYSMICON,
            ),
            load_icon(
                include_bytes!(concat!(env!("OUT_DIR"), "/StatusEng10.ico")),
                SM_CXSMICON,
                SM_CYSMICON,
            ),
        ])
    }

    pub fn app(&self) -> HICON {
        self.0[0]
    }

    pub fn language(&self, vietnamese: bool, gray: bool) -> HICON {
        let icon = self.0[1 + usize::from(!vietnamese) + 2 * usize::from(gray)];
        if icon.is_null() {
            self.app()
        } else {
            icon
        }
    }
}

impl Drop for Icons {
    fn drop(&mut self) {
        for icon in self.0 {
            if !icon.is_null() {
                unsafe { DestroyIcon(icon) };
            }
        }
    }
}

#[cfg(not(target_env = "msvc"))]
unsafe fn load_icon(file: &[u8], x: i32, y: i32) -> HICON {
    // ICO directory entries use offsets. The Win32 resource selector takes integer IDs.
    let count = u16::from_le_bytes([file[4], file[5]]);
    let mut directory = file[..6].to_vec();
    for (index, entry) in file[6..6 + usize::from(count) * 16]
        .as_chunks::<16>()
        .0
        .iter()
        .enumerate()
    {
        directory.extend_from_slice(&entry[..12]);
        directory.extend_from_slice(&((index + 1) as u16).to_le_bytes());
    }
    let width = GetSystemMetrics(x);
    let height = GetSystemMetrics(y);
    let id = LookupIconIdFromDirectoryEx(directory.as_ptr(), 1, width, height, LR_DEFAULTCOLOR);
    if id <= 0 || id > i32::from(count) {
        return std::ptr::null_mut();
    }
    let entry = &file[6 + (id as usize - 1) * 16..];
    let length = u32::from_le_bytes(entry[8..12].try_into().unwrap());
    let offset = u32::from_le_bytes(entry[12..16].try_into().unwrap()) as usize;
    let Some(bits) = file.get(offset..offset + length as usize) else {
        return std::ptr::null_mut();
    };
    let mut aligned = vec![0u32; bits.len().div_ceil(4)];
    std::ptr::copy_nonoverlapping(bits.as_ptr(), aligned.as_mut_ptr().cast(), bits.len());
    CreateIconFromResourceEx(
        aligned.as_ptr().cast(),
        length,
        1,
        0x0003_0000,
        width,
        height,
        LR_DEFAULTCOLOR,
    )
}

pub unsafe fn update(owner: HWND, command: NOTIFY_ICON_MESSAGE) -> bool {
    let Some((viet, icon)) = read_app(|app| {
        let preferences = &app.typing.preferences;
        let viet = preferences.enabled(Setting::Language);
        (
            viet,
            app.icons
                .language(viet, preferences.enabled(Setting::GrayIcon)),
        )
    }) else {
        return false;
    };
    let mut data: NOTIFYICONDATAW = zeroed();
    data.cbSize = size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = owner;
    data.uID = 100;
    data.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    data.uCallbackMessage = WM_TRAY;
    data.hIcon = icon;
    let tip = wide(if viet {
        "NeoKey – Tiếng Việt"
    } else {
        "NeoKey – Tiếng Anh"
    });
    data.szTip[..tip.len()].copy_from_slice(&tip);
    Shell_NotifyIconW(command, &data) != 0
}

fn menu_item(menu: HMENU, id: u16, label: &str, checked: bool) {
    unsafe {
        AppendMenuW(
            menu,
            if checked { MF_CHECKED } else { MF_STRING },
            id.into(),
            wide(label).as_ptr(),
        );
    }
}

pub fn hotkey_name(hotkey: Hotkey) -> String {
    let mut parts = Vec::new();
    for (bit, name) in [(CTRL, "Ctrl"), (ALT, "Alt"), (WIN, "Win"), (SHIFT, "Shift")] {
        if hotkey.mods() & bit != 0 {
            parts.push(name.to_owned());
        }
    }
    let key = key_name(hotkey.vk());
    if !key.is_empty() {
        parts.push(key);
    }
    parts.join(" + ")
}

pub fn key_name(vk: u8) -> String {
    if vk == 0 || vk == Hotkey::MODIFIERS_ONLY {
        return String::new();
    }
    if vk == 0x20 {
        return "Space".to_owned();
    }
    unsafe {
        use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
            GetKeyNameTextW, MapVirtualKeyW, MAPVK_VK_TO_VSC_EX,
        };
        let scan = MapVirtualKeyW(vk.into(), MAPVK_VK_TO_VSC_EX);
        let param = ((scan & 0xff) << 16) | (u32::from(scan > 0xff) << 24);
        let mut buffer = [0; 64];
        let len = GetKeyNameTextW(param as i32, buffer.as_mut_ptr(), buffer.len() as i32);
        if len <= 0 {
            format!("VK {vk:02X}")
        } else {
            String::from_utf16_lossy(&buffer[..len as usize])
        }
    }
}

pub unsafe fn popup(owner: HWND) -> u16 {
    use Setting::*;
    let Some(preferences) = read_app(|app| app.typing.preferences.clone()) else {
        return 0;
    };
    let menu = CreatePopupMenu();
    if menu.is_null() {
        return 0;
    }
    let separator = || {
        AppendMenuW(menu, MF_SEPARATOR, 0, null());
    };
    menu_item(
        menu,
        TOGGLE,
        "Bật Tiếng Việt",
        preferences.enabled(Language),
    );
    separator();
    menu_item(
        menu,
        SPELLING,
        "Bật kiểm tra chính tả",
        preferences.enabled(Spelling),
    );
    menu_item(
        menu,
        SMART_SWITCH,
        "Bật loại trừ ứng dụng thông minh",
        preferences.enabled(SmartSwitch),
    );
    menu_item(menu, USE_MACRO, "Bật gõ tắt", preferences.enabled(UseMacro));
    separator();
    menu_item(menu, MACROS, "Cấu hình gõ tắt...", false);
    menu_item(menu, CONVERT_TOOL, "Công cụ chuyển mã...", false);
    let name = hotkey_name(Hotkey(preferences.get(ConvertKey)));
    let title = if name.is_empty() {
        "Chuyển mã nhanh".to_owned()
    } else {
        format!("Chuyển mã nhanh - [{name}]")
    };
    menu_item(menu, QUICK_CONVERT, &title, false);
    separator();
    for (index, name) in INPUT_TYPES.iter().enumerate() {
        menu_item(
            menu,
            INPUT_TYPE + index as u16,
            &format!("Kiểu gõ {name}"),
            preferences.get(InputType) == index as u32,
        );
    }
    separator();
    let other = CreatePopupMenu();
    for (index, name) in CODE_TABLES.iter().enumerate() {
        let parent = if index >= 3 && !other.is_null() {
            other
        } else {
            menu
        };
        menu_item(
            parent,
            CODE_TABLE + index as u16,
            name,
            preferences.get(CodeTable) == index as u32,
        );
    }
    if !other.is_null()
        && AppendMenuW(
            menu,
            MF_POPUP,
            other as usize,
            windows_sys::w!("Bảng mã khác"),
        ) == 0
    {
        DestroyMenu(other);
    }
    separator();
    menu_item(menu, CONTROL_PANEL, "Bảng điều khiển...", false);
    menu_item(menu, ABOUT, "Giới thiệu NeoKey", false);
    separator();
    menu_item(menu, EXIT, "Thoát", false);
    SetMenuDefaultItem(menu, CONTROL_PANEL.into(), 0);

    let mut point = POINT { x: 0, y: 0 };
    GetCursorPos(&mut point);
    SetForegroundWindow(owner);
    let selected = TrackPopupMenu(
        menu,
        TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON,
        point.x,
        point.y,
        0,
        owner,
        null(),
    ) as u16;
    PostMessageW(owner, WM_NULL, 0, 0);
    DestroyMenu(menu);
    selected
}
