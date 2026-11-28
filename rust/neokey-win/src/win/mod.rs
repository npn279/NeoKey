mod clipboard;
mod hooks;
mod registry;
mod system;
mod tray;
mod ui;

use neokey_win::{
    data::RELEASES_URL,
    settings::{Setting, MACRO_DATA, SMART_SWITCH_DATA},
    typing::Typing,
};
use std::{
    cell::RefCell,
    io,
    ptr::{null, null_mut},
    sync::mpsc::Receiver,
};
use windows_sys::{
    w,
    Win32::{
        Foundation::*,
        System::{
            Diagnostics::Debug::MessageBeep,
            LibraryLoader::GetModuleHandleW,
            Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject},
        },
        UI::{
            Shell::{IsUserAnAdmin, NIM_ADD, NIM_DELETE, NIM_MODIFY},
            WindowsAndMessaging::*,
        },
    },
};

const OWNER_CLASS: *const u16 = w!("OpenKeyVietnameseInputMethod");
const SHOW_EXISTING: u32 = WM_USER + 2019;
const WM_TRAY: u32 = WM_APP + 1;
const WM_FOREGROUND: u32 = WM_APP + 2;
const WM_LANGUAGE_CHANGED: u32 = WM_APP + 3;
const WM_CONVERT: u32 = WM_APP + 4;
const WM_UPDATE_DONE: u32 = WM_APP + 5;
const STARTUP_WORK: u32 = WM_APP + 6;
const STARTUP_DONE: u32 = WM_APP + 7;

struct App {
    owner: HWND,
    registry: registry::Registry,
    typing: Typing,
    icons: tray::Icons,
    taskbar_message: u32,
    last_pid: u32,
    foreground_name: String,
    persisted: neokey_win::settings::Preferences,
    smart_dirty: bool,
    update: Option<(bool, Receiver<system::UpdateResult>)>,
    startup: Option<Receiver<io::Result<()>>>,
    startup_pending: bool,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

fn with_app<R>(operation: impl FnOnce(&mut App) -> R) -> Option<R> {
    APP.with(|app| app.try_borrow_mut().ok()?.as_mut().map(operation))
}

fn read_app<R>(operation: impl FnOnce(&App) -> R) -> Option<R> {
    APP.with(|app| app.try_borrow().ok()?.as_ref().map(operation))
}

impl App {
    fn toggle_language(&mut self) {
        let new = u32::from(!self.typing.preferences.enabled(Setting::Language));
        self.change(Setting::Language, new);
    }

    fn change(&mut self, setting: Setting, value: u32) {
        if self.typing.preferences.get(setting) == value {
            return;
        }
        self.typing.preferences.set(setting, value);
        self.typing.refresh_settings();
        if (setting == Setting::Language && self.typing.preferences.enabled(Setting::SmartSwitch))
            || (setting == Setting::CodeTable
                && self.typing.preferences.enabled(Setting::RememberTable))
        {
            self.smart_dirty |= self.typing.remember_app();
        }
    }

    fn save(&mut self) -> io::Result<()> {
        for &setting in Setting::ALL {
            let new = self.typing.preferences.get(setting);
            if new != self.persisted.get(setting) {
                self.registry.write_setting(setting, new)?;
                self.persisted.set(setting, new);
            }
        }
        if self.smart_dirty {
            self.registry
                .write_binary(SMART_SWITCH_DATA, &self.typing.smart_switch.save_data())?;
            self.smart_dirty = false;
        }
        Ok(())
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

unsafe fn notice(owner: HWND, message: &str, flags: u32) -> i32 {
    MessageBoxW(owner, wide(message).as_ptr(), w!("NeoKey"), flags)
}

unsafe fn error(owner: HWND, message: &str) {
    notice(owner, message, MB_OK | MB_ICONERROR);
}

unsafe fn save_and_refresh(owner: HWND) {
    let result = with_app(App::save);
    tray::update(owner, NIM_MODIFY);
    ui::refresh();
    if let Some(Err(err)) = result {
        error(owner, &format!("Không lưu được cài đặt.\n{err}"));
    }
}

unsafe fn change(owner: HWND, setting: Setting, value: u32) {
    with_app(|app| app.change(setting, value));
    save_and_refresh(owner);
}

unsafe fn foreground(owner: HWND) {
    let window = GetForegroundWindow();
    let pid = system::window_pid(window);
    let lookup = read_app(|app| app.last_pid != pid).unwrap_or(false);
    let name = if lookup {
        system::executable_name(pid)
    } else {
        None
    };
    let Some((refresh, metro)) = with_app(|app| {
        let before_language = app.typing.preferences.get(Setting::Language);
        let before_table = app.typing.preferences.get(Setting::CodeTable);
        if lookup {
            app.last_pid = pid;
            app.foreground_name = name.unwrap_or_default();
        }
        if pid == std::process::id() {
            app.typing.reset();
        } else {
            app.smart_dirty |= app.typing.foreground(&app.foreground_name);
        }
        let refresh = app.smart_dirty
            || before_language != app.typing.preferences.get(Setting::Language)
            || before_table != app.typing.preferences.get(Setting::CodeTable);
        let preferences = &app.typing.preferences;
        let metro = app
            .foreground_name
            .eq_ignore_ascii_case("ApplicationFrameHost.exe")
            && app.typing.metro()
            && (preferences.enabled(Setting::SmartSwitch)
                || preferences.enabled(Setting::RememberTable));
        (refresh, metro)
    }) else {
        return;
    };
    if refresh {
        save_and_refresh(owner);
    }
    if metro {
        hooks::metro_backspace();
    }
}

unsafe fn convert(owner: HWND, dialog: HWND) {
    let Some(options) = read_app(|app| app.typing.preferences.convert()) else {
        return;
    };
    match clipboard::convert(owner, &options) {
        Ok(()) if !options.dont_alert_when_completed => {
            notice(
                dialog,
                "Chuyển mã thành công. Kết quả nằm trong Clipboard.",
                MB_OK | MB_ICONINFORMATION,
            );
        }
        Err(err) => error(dialog, &err),
        _ => {}
    }
}

unsafe fn begin_update(owner: HWND, manual: bool) {
    let result = with_app(|app| {
        if let Some((show, _)) = &mut app.update {
            *show |= manual;
            return Ok(());
        }
        app.update = Some((manual, system::check_update(owner)?));
        Ok::<_, io::Error>(())
    });
    if manual {
        if let Some(Err(err)) = result {
            error(owner, &format!("Không kiểm tra được bản mới.\n{err}"));
        }
    }
}

unsafe fn finish_update(owner: HWND) {
    let Some(Some((manual, receiver))) = with_app(|app| app.update.take()) else {
        return;
    };
    match receiver.try_recv() {
        Ok(Ok(Some(version))) => {
            if notice(
                owner,
                &format!("Đã có phiên bản {version}. Mở trang tải về?"),
                MB_YESNO | MB_ICONQUESTION,
            ) == IDYES
            {
                system::shell_open(owner, &wide(RELEASES_URL));
            }
        }
        Ok(Ok(None)) if manual => {
            notice(
                owner,
                "Bạn đang dùng phiên bản mới nhất!",
                MB_OK | MB_ICONINFORMATION,
            );
        }
        Ok(Err(err)) if manual => error(owner, &format!("Không kiểm tra được bản mới.\n{err}")),
        _ => {}
    }
}

unsafe fn begin_startup_work(owner: HWND) {
    let result = with_app(|app| {
        if app.startup.is_some() {
            app.startup_pending = true;
            return Ok(());
        }
        let run = app.typing.preferences.enabled(Setting::RunWithWindows);
        let admin = app.typing.preferences.enabled(Setting::Administrator);
        let (send, receive) = std::sync::mpsc::channel();
        let owner = owner as usize;
        std::thread::Builder::new()
            .name("startup-settings".into())
            .spawn(move || {
                let result = system::configure_startup(run, admin);
                if send.send(result).is_ok() {
                    PostMessageW(owner as HWND, STARTUP_DONE, 0, 0);
                }
            })?;
        app.startup = Some(receive);
        Ok::<_, io::Error>(())
    });
    if let Some(Err(err)) = result {
        error(
            owner,
            &format!("Không cập nhật được tùy chọn khởi động.\n{err}"),
        );
    }
}

unsafe fn finish_startup_work(owner: HWND) {
    let Some((pending, result)) = with_app(|app| {
        (
            std::mem::take(&mut app.startup_pending),
            app.startup.take().and_then(|rx| rx.try_recv().ok()),
        )
    }) else {
        return;
    };
    if pending {
        begin_startup_work(owner);
    } else if let Some(Err(err)) = result {
        error(
            owner,
            &format!("Không cập nhật được tùy chọn khởi động.\n{err}"),
        );
    }
}

unsafe fn tray_command(owner: HWND, id: u16) {
    match id {
        tray::TOGGLE => {
            // The tray and the control-panel language control do not beep.
            // Only the language shortcut does, from WM_LANGUAGE_CHANGED.
            with_app(|app| app.toggle_language());
            save_and_refresh(owner);
        }
        tray::SPELLING | tray::SMART_SWITCH | tray::USE_MACRO => {
            let setting = match id {
                tray::SPELLING => Setting::Spelling,
                tray::SMART_SWITCH => Setting::SmartSwitch,
                _ => Setting::UseMacro,
            };
            let value = read_app(|app| !app.typing.preferences.enabled(setting)).unwrap_or(false);
            change(owner, setting, value.into());
        }
        tray::INPUT_TYPE..=913 => change(owner, Setting::InputType, (id - tray::INPUT_TYPE).into()),
        tray::CODE_TABLE..=934 => change(owner, Setting::CodeTable, (id - tray::CODE_TABLE).into()),
        tray::QUICK_CONVERT => convert(owner, null_mut()),
        tray::EXIT => PostQuitMessage(0),
        tray::CONTROL_PANEL => ui::show(owner, ui::Panel::Main),
        tray::CONVERT_TOOL => ui::show(owner, ui::Panel::Convert),
        tray::MACROS => ui::show(owner, ui::Panel::Macros),
        tray::ABOUT => ui::show(owner, ui::Panel::About),
        _ => {}
    }
}

unsafe extern "system" fn owner_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        SHOW_EXISTING => ui::show(window, ui::Panel::Main),
        WM_TRAY => match lparam as u32 {
            WM_LBUTTONUP => tray_command(window, tray::TOGGLE),
            WM_LBUTTONDBLCLK => ui::show(window, ui::Panel::Main),
            WM_RBUTTONDOWN => tray_command(window, tray::popup(window)),
            _ => {}
        },
        WM_FOREGROUND => foreground(window),
        WM_CONVERT => convert(window, null_mut()),
        WM_LANGUAGE_CHANGED => {
            if wparam != 0 {
                windows_sys::Win32::System::Diagnostics::Debug::MessageBeep(MB_OK);
            }
            save_and_refresh(window);
        }
        WM_UPDATE_DONE => finish_update(window),
        STARTUP_WORK => begin_startup_work(window),
        STARTUP_DONE => finish_startup_work(window),
        WM_CLOSE => PostQuitMessage(0),
        _ => {
            if read_app(|app| app.taskbar_message == message && message != 0).unwrap_or(false) {
                tray::update(window, NIM_ADD);
            }
            return DefWindowProcW(window, message, wparam, lparam);
        }
    }
    0
}

struct Instance(HANDLE);

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            ReleaseMutex(self.0);
            CloseHandle(self.0);
        }
    }
}

unsafe fn start() -> io::Result<()> {
    let registry = registry::Registry::open()?;
    let preferences = registry.preferences();
    if preferences.enabled(Setting::Administrator) && IsUserAnAdmin() == 0 {
        system::relaunch_elevated(null_mut(), false)?;
        return Ok(());
    }

    let existing = FindWindowW(OWNER_CLASS, null());
    let waiting = std::env::args_os().any(|arg| arg == "--wait-for-previous");
    if !waiting && !existing.is_null() {
        MessageBeep(MB_OK);
        PostMessageW(existing, SHOW_EXISTING, 0, 0);
        return Ok(());
    }
    let lock = CreateMutexW(null(), 0, w!("Local\\OpenKeyVietnameseInputMethod"));
    if lock.is_null() {
        return Err(io::Error::last_os_error());
    }
    if !matches!(
        WaitForSingleObject(lock, if waiting { 12_000 } else { 0 }),
        WAIT_OBJECT_0 | WAIT_ABANDONED
    ) {
        CloseHandle(lock);
        let existing = FindWindowW(OWNER_CLASS, null());
        if !existing.is_null() {
            MessageBeep(MB_OK);
            PostMessageW(existing, SHOW_EXISTING, 0, 0);
        }
        return Ok(());
    }
    let _instance = Instance(lock);
    let module = GetModuleHandleW(null());
    let icons = tray::Icons::load();
    let mut class: WNDCLASSW = std::mem::zeroed();
    class.lpfnWndProc = Some(owner_proc);
    class.hInstance = module;
    class.hIcon = icons.app();
    class.lpszClassName = OWNER_CLASS;
    if RegisterClassW(&class) == 0 {
        return Err(io::Error::last_os_error());
    }
    ui::register(module, icons.app())?;
    let owner = CreateWindowExW(
        WS_EX_TOOLWINDOW,
        OWNER_CLASS,
        w!("NeoKey"),
        WS_OVERLAPPED,
        0,
        0,
        0,
        0,
        null_mut(),
        null_mut(),
        module,
        null(),
    );
    if owner.is_null() {
        return Err(io::Error::last_os_error());
    }
    let mut typing = Typing::new(preferences.clone());
    let macro_error = typing
        .engine
        .init_macros(&registry.read_binary(MACRO_DATA))
        .is_err();
    let mut smart = neokey_engine::SmartSwitch::default();
    if smart.init(&registry.read_binary(SMART_SWITCH_DATA)).is_ok() {
        typing.smart_switch = smart;
    }
    let show = preferences.enabled(Setting::ShowAtStartup);
    let update = preferences.enabled(Setting::CheckUpdate);
    APP.with(|app| {
        *app.borrow_mut() = Some(App {
            owner,
            typing,
            registry,
            icons,
            taskbar_message: RegisterWindowMessageW(w!("TaskbarCreated")),
            last_pid: 0,
            foreground_name: String::new(),
            persisted: preferences,
            smart_dirty: false,
            update: None,
            startup: None,
            startup_pending: false,
        })
    });
    let hooks = hooks::Hooks::install(module)?;
    let added = tray::update(owner, NIM_ADD);
    foreground(owner);
    PostMessageW(owner, STARTUP_WORK, 0, 0);
    if show || !added {
        ui::show(owner, ui::Panel::Main);
    }
    MessageBeep(MB_OK);
    if update {
        begin_update(owner, false);
    }
    if macro_error {
        error(
            owner,
            "Không đọc được tất cả dữ liệu gõ tắt trong registry.",
        );
    }

    let mut message: MSG = std::mem::zeroed();
    let result = loop {
        match GetMessageW(&mut message, null_mut(), 0, 0) {
            -1 => break Err(io::Error::last_os_error()),
            0 => break Ok(()),
            _ if ui::handle_message(&message) => {}
            _ => {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
    };
    drop(hooks);
    with_app(App::save);
    tray::update(owner, NIM_DELETE);
    ui::close_all();
    DestroyWindow(owner);
    APP.with(|app| app.borrow_mut().take());
    result
}

pub fn run() {
    unsafe {
        if let Err(err) = start() {
            error(null_mut(), &format!("Không khởi động được NeoKey.\n{err}"));
        }
    }
}
