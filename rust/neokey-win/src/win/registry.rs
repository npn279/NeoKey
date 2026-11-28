use super::wide;
use neokey_win::settings::{Preferences, Setting, REGISTRY_PATH, RUN_PATH, RUN_VALUE};
use std::{
    io,
    mem::size_of,
    os::windows::ffi::OsStrExt,
    path::Path,
    ptr::{null, null_mut},
};
use windows_sys::Win32::{Foundation::ERROR_SUCCESS, System::Registry::*};

const MAX_BINARY: u32 = 16 * 1024 * 1024;

pub struct Registry(HKEY);

impl Registry {
    pub fn open() -> io::Result<Self> {
        Self::at(REGISTRY_PATH)
    }

    fn at(path: &str) -> io::Result<Self> {
        let mut key = null_mut();
        let result = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                wide(path).as_ptr(),
                0,
                null(),
                REG_OPTION_NON_VOLATILE,
                KEY_QUERY_VALUE | KEY_SET_VALUE,
                null(),
                &mut key,
                null_mut(),
            )
        };
        if result == ERROR_SUCCESS {
            Ok(Self(key))
        } else {
            Err(io::Error::from_raw_os_error(result as i32))
        }
    }

    pub fn preferences(&self) -> Preferences {
        Preferences::load(|name| {
            let mut value = 0u32;
            let mut bytes = size_of::<u32>() as u32;
            let result = unsafe {
                RegGetValueW(
                    self.0,
                    null(),
                    wide(name).as_ptr(),
                    RRF_RT_REG_DWORD,
                    null_mut(),
                    (&mut value as *mut u32).cast(),
                    &mut bytes,
                )
            };
            (result == ERROR_SUCCESS).then_some(value)
        })
    }

    pub fn read_binary(&self, name: &str) -> Vec<u8> {
        let name = wide(name);
        let mut size = 0;
        unsafe {
            if RegGetValueW(
                self.0,
                null(),
                name.as_ptr(),
                RRF_RT_REG_BINARY,
                null_mut(),
                null_mut(),
                &mut size,
            ) != ERROR_SUCCESS
                || size > MAX_BINARY
            {
                return Vec::new();
            }
            let mut value = vec![0; size as usize];
            if RegGetValueW(
                self.0,
                null(),
                name.as_ptr(),
                RRF_RT_REG_BINARY,
                null_mut(),
                value.as_mut_ptr().cast(),
                &mut size,
            ) != ERROR_SUCCESS
            {
                return Vec::new();
            }
            value.truncate(size as usize);
            value
        }
    }

    pub fn write_binary(&self, name: &str, value: &[u8]) -> io::Result<()> {
        if value.len() > MAX_BINARY as usize {
            return Err(io::Error::other("Dữ liệu vượt giới hạn lưu trữ."));
        }
        self.write(name, REG_BINARY, value)
    }

    pub fn write_setting(&self, setting: Setting, value: u32) -> io::Result<()> {
        self.write(setting.key(), REG_DWORD, &value.to_le_bytes())
    }

    fn write(&self, name: &str, ty: u32, value: &[u8]) -> io::Result<()> {
        let status = unsafe {
            RegSetValueExW(
                self.0,
                wide(name).as_ptr(),
                0,
                ty,
                value.as_ptr(),
                value.len() as u32,
            )
        };
        if status == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(status as i32))
        }
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        unsafe { RegCloseKey(self.0) };
    }
}

pub fn set_run_value(exe: Option<&Path>) -> io::Result<()> {
    let key = Registry::at(RUN_PATH)?;
    if let Some(exe) = exe {
        let quote = u16::from(b'"');
        let bytes: Vec<_> = std::iter::once(quote)
            .chain(exe.as_os_str().encode_wide())
            .chain([quote, 0])
            .flat_map(u16::to_le_bytes)
            .collect();
        key.write(RUN_VALUE, REG_SZ, &bytes)
    } else {
        let code = unsafe { RegDeleteValueW(key.0, wide(RUN_VALUE).as_ptr()) };
        if code == ERROR_SUCCESS || code == windows_sys::Win32::Foundation::ERROR_FILE_NOT_FOUND {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(code as i32))
        }
    }
}
