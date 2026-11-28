use super::{registry, wide, WM_UPDATE_DONE};
use neokey_win::data::{available_version, current_version_code, UPDATE_URL};
use std::{
    ffi::{c_void, OsStr, OsString},
    io,
    os::windows::{
        ffi::{OsStrExt, OsStringExt},
        process::CommandExt,
    },
    path::{Path, PathBuf},
    ptr::{null, null_mut},
    sync::mpsc,
};
use windows_sys::{
    w,
    Win32::{
        Foundation::{CloseHandle, HWND},
        Networking::WinHttp::*,
        System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
        },
        UI::{
            Shell::{IsUserAnAdmin, SHGetFolderPathW, ShellExecuteW, CSIDL_DESKTOPDIRECTORY},
            WindowsAndMessaging::{GetWindowThreadProcessId, PostMessageW, SW_SHOWNORMAL},
        },
    },
};

pub unsafe fn executable_name(pid: u32) -> Option<String> {
    if pid == 0 || pid == std::process::id() {
        return None;
    }
    let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
    if process.is_null() {
        return None;
    }
    let mut buffer = [0; 32768];
    let mut length = buffer.len() as u32;
    let result = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut length);
    CloseHandle(process);
    if result == 0 {
        return None;
    }
    let path = String::from_utf16_lossy(&buffer[..length as usize]);
    path.rsplit('\\').next().map(str::to_owned)
}

pub unsafe fn window_pid(window: HWND) -> u32 {
    let mut pid = 0;
    GetWindowThreadProcessId(window, &mut pid);
    pid
}

pub unsafe fn shell_open(owner: HWND, path: &[u16]) -> bool {
    ShellExecuteW(
        owner,
        w!("open"),
        path.as_ptr(),
        null(),
        null(),
        SW_SHOWNORMAL,
    ) as isize
        > 32
}

pub fn path_wide(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain(Some(0)).collect()
}

pub unsafe fn relaunch_elevated(owner: HWND, wait: bool) -> io::Result<()> {
    let path = path_wide(&std::env::current_exe()?);
    let args = if wait {
        w!("--wait-for-previous")
    } else {
        null()
    };
    if ShellExecuteW(
        owner,
        w!("runas"),
        path.as_ptr(),
        args,
        null(),
        SW_SHOWNORMAL,
    ) as isize
        > 32
    {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

pub unsafe fn configure_startup(run: bool, elevate: bool) -> io::Result<()> {
    let admin = IsUserAnAdmin() != 0;
    if run && elevate && admin {
        let exe = std::env::current_exe()?;
        let mut quoted = OsString::from("\"");
        quoted.push(exe);
        quoted.push("\"");
        let mut args = [
            "/create", "/sc", "onlogon", "/tn", "OpenKey", "/rl", "highest", "/tr", "", "/f",
        ]
        .map(OsStr::new);
        args[8] = &quoted;
        if !scheduled_task(&args)? {
            return Err(io::Error::other("Không tạo được tác vụ khởi động OpenKey."));
        }
        registry::set_run_value(None)
    } else {
        let exe = std::env::current_exe()?;
        registry::set_run_value(run.then_some(exe.as_path()))?;
        if admin || !run {
            scheduled_task(&["/delete", "/tn", "OpenKey", "/f"].map(OsStr::new))?;
        }
        Ok(())
    }
}

fn scheduled_task(args: &[&OsStr]) -> io::Result<bool> {
    // Resolve schtasks from the system directory even when running elevated.
    #[link(name = "kernel32")]
    extern "system" {
        fn GetSystemDirectoryW(buffer: *mut u16, capacity: u32) -> u32;
    }
    let mut buffer = [0; 32768];
    let length = unsafe { GetSystemDirectoryW(buffer.as_mut_ptr(), buffer.len() as u32) };
    if length == 0 || length >= buffer.len() as u32 {
        return Err(io::Error::last_os_error());
    }
    let exe = PathBuf::from(OsString::from_wide(&buffer[..length as usize])).join("schtasks.exe");
    let mut child = std::process::Command::new(exe)
        .creation_flags(windows_sys::Win32::System::Threading::CREATE_NO_WINDOW)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait()? {
            return Ok(status.success());
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(io::Error::new(io::ErrorKind::TimedOut, "schtasks.exe"));
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

pub unsafe fn create_shortcut() -> windows::core::Result<()> {
    use windows::{
        core::{Interface, PCWSTR},
        Win32::{
            System::Com::{
                CoCreateInstance, CoInitializeEx, CoUninitialize, IPersistFile,
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
            },
            UI::Shell::{IShellLinkW, ShellLink},
        },
    };
    CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
    struct Apartment;
    impl Drop for Apartment {
        fn drop(&mut self) {
            unsafe { CoUninitialize() };
        }
    }
    let _apartment = Apartment;
    let link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
    let exe = std::env::current_exe().map_err(|_| windows::core::Error::from_win32())?;
    let target = path_wide(&exe);
    link.SetPath(PCWSTR(target.as_ptr()))?;
    link.SetDescription(windows::core::w!("NeoKey - Bộ gõ tiếng Việt"))?;
    link.SetIconLocation(PCWSTR(target.as_ptr()), 0)?;
    let mut desktop = [0; 260];
    windows::core::HRESULT(SHGetFolderPathW(
        null_mut(),
        CSIDL_DESKTOPDIRECTORY as i32,
        null_mut(),
        0,
        desktop.as_mut_ptr(),
    ))
    .ok()?;
    let length = desktop
        .iter()
        .position(|&unit| unit == 0)
        .unwrap_or(desktop.len());
    let path = PathBuf::from(OsString::from_wide(&desktop[..length])).join("OpenKey.lnk");
    link.cast::<IPersistFile>()?
        .Save(PCWSTR(path_wide(&path).as_ptr()), true)?;
    Ok(())
}

pub type UpdateResult = Result<Option<String>, String>;

pub fn check_update(owner: HWND) -> io::Result<mpsc::Receiver<UpdateResult>> {
    let (tx, rx) = mpsc::channel();
    let owner = owner as usize;
    std::thread::Builder::new()
        .name("update-check".into())
        .spawn(move || {
            let result = unsafe { download_manifest() }.and_then(|body| {
                available_version(&body, current_version_code()).map_err(str::to_owned)
            });
            if tx.send(result).is_ok() {
                unsafe { PostMessageW(owner as HWND, WM_UPDATE_DONE, 0, 0) };
            }
        })?;
    Ok(rx)
}

struct Internet(*mut c_void);

impl Internet {
    fn new(handle: *mut c_void) -> Result<Self, String> {
        if handle.is_null() {
            Err(io::Error::last_os_error().to_string())
        } else {
            Ok(Self(handle))
        }
    }
}

impl Drop for Internet {
    fn drop(&mut self) {
        unsafe { WinHttpCloseHandle(self.0) };
    }
}

unsafe fn download_manifest() -> Result<Vec<u8>, String> {
    let url = UPDATE_URL
        .strip_prefix("https://")
        .expect("HTTPS update URL");
    let (host, path) = url.split_once('/').expect("update URL includes a path");
    let session = Internet::new(WinHttpOpen(
        w!("NeoKey Win"),
        WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY,
        null(),
        null(),
        0,
    ))?;
    let status = |ok| {
        if ok != 0 {
            Ok(())
        } else {
            Err(io::Error::last_os_error().to_string())
        }
    };
    status(WinHttpSetTimeouts(session.0, 4000, 4000, 4000, 4000))?;
    let connection = Internet::new(WinHttpConnect(session.0, wide(host).as_ptr(), 443, 0))?;
    let request = Internet::new(WinHttpOpenRequest(
        connection.0,
        w!("GET"),
        wide(&format!("/{path}")).as_ptr(),
        null(),
        null(),
        null(),
        WINHTTP_FLAG_SECURE | WINHTTP_FLAG_REFRESH,
    ))?;
    let redirect = WINHTTP_OPTION_REDIRECT_POLICY_NEVER;
    status(WinHttpSetOption(
        request.0,
        WINHTTP_OPTION_REDIRECT_POLICY,
        (&redirect as *const u32).cast(),
        4,
    ))?;
    status(WinHttpSendRequest(request.0, null(), 0, null(), 0, 0, 0))?;
    status(WinHttpReceiveResponse(request.0, null_mut()))?;
    let mut code = 0u32;
    let mut size = 4;
    status(WinHttpQueryHeaders(
        request.0,
        WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER,
        null(),
        (&mut code as *mut u32).cast(),
        &mut size,
        null_mut(),
    ))?;
    if code != 200 {
        return Err(format!("HTTP {code}"));
    }
    let mut body = Vec::new();
    let mut buffer = [0; 4096];
    for _ in 0..5 {
        let mut received = 0;
        status(WinHttpReadData(
            request.0,
            buffer.as_mut_ptr().cast(),
            buffer.len() as u32,
            &mut received,
        ))?;
        if received == 0 {
            return Ok(body);
        }
        body.extend_from_slice(&buffer[..received as usize]);
        if body.len() > 16 * 1024 {
            break;
        }
    }
    Err("Tệp phiên bản quá lớn.".into())
}
