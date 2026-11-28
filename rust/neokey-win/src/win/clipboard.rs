use neokey_engine::ConvertOptions;
use std::{mem::size_of, ptr, slice};
use windows_sys::Win32::{
    Foundation::HWND,
    System::{DataExchange::*, Memory::*, Ole::CF_UNICODETEXT},
};

const MAX_CLIPBOARD_BYTES: usize = 16 * 1024 * 1024;

struct Clipboard;

impl Clipboard {
    unsafe fn open(owner: HWND) -> Option<Self> {
        (OpenClipboard(owner) != 0).then_some(Self)
    }

    unsafe fn read<T: Copy>(&self, format: u32) -> Option<Vec<T>> {
        let handle = GetClipboardData(format);
        if handle.is_null() {
            return None;
        }
        let size = GlobalSize(handle);
        if size == 0 || size > MAX_CLIPBOARD_BYTES {
            return None;
        }
        let source = GlobalLock(handle).cast::<T>();
        if source.is_null() {
            return None;
        }
        let output = slice::from_raw_parts(source, size / size_of::<T>()).to_vec();
        GlobalUnlock(handle);
        Some(output)
    }

    unsafe fn replace(&self, entries: &[(u32, Block)]) -> bool {
        if EmptyClipboard() == 0 {
            return false;
        }
        let mut complete = true;
        for (format, block) in entries {
            if SetClipboardData(*format, block.handle).is_null() {
                complete = false;
            } else {
                block.transferred.set(true);
            }
        }
        complete
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        unsafe { CloseClipboard() };
    }
}

struct Block {
    handle: windows_sys::Win32::Foundation::HGLOBAL,
    transferred: std::cell::Cell<bool>,
}

impl Block {
    unsafe fn new<T: Copy + Default>(data: &[T]) -> Option<Self> {
        let size = (data.len() + 1).checked_mul(size_of::<T>())?;
        if size > MAX_CLIPBOARD_BYTES {
            return None;
        }
        let handle = GlobalAlloc(GMEM_MOVEABLE, size);
        if handle.is_null() {
            return None;
        }
        let block = Self {
            handle,
            transferred: false.into(),
        };
        let target = GlobalLock(handle).cast::<T>();
        if target.is_null() {
            return None;
        }
        ptr::copy_nonoverlapping(data.as_ptr(), target, data.len());
        target.add(data.len()).write(T::default());
        GlobalUnlock(handle);
        Some(block)
    }
}

impl Drop for Block {
    fn drop(&mut self) {
        if !self.transferred.get() {
            unsafe { windows_sys::Win32::Foundation::GlobalFree(self.handle) };
        }
    }
}

pub unsafe fn paste_text(owner: HWND, units: &[u16]) -> bool {
    if units.contains(&0) {
        return false;
    }
    let Some(block) = Block::new(units) else {
        return false;
    };
    let Some(clipboard) = Clipboard::open(owner) else {
        return false;
    };
    clipboard.replace(&[(u32::from(CF_UNICODETEXT), block)])
}

fn nul_prefix<T: Default + PartialEq>(buffer: &[T]) -> &[T] {
    let len = buffer
        .iter()
        .position(|item| *item == T::default())
        .unwrap_or(buffer.len());
    &buffer[..len]
}

pub unsafe fn convert(owner: HWND, options: &ConvertOptions) -> Result<(), String> {
    let unicode = u32::from(CF_UNICODETEXT);
    let html = RegisterClipboardFormatW(windows_sys::w!("HTML Format"));
    let clipboard = Clipboard::open(owner).ok_or("Không mở được Clipboard.")?;

    let mut entries = Vec::new();
    if let Some(text) = clipboard.read::<u16>(unicode) {
        let source = String::from_utf16(nul_prefix(&text)).map_err(|error| error.to_string())?;
        let text: Vec<u16> = options
            .convert(&source)
            .map_err(|error| error.to_string())?
            .encode_utf16()
            .collect();
        entries.push((unicode, Block::new(&text).ok_or("Không lưu được văn bản.")?));
    }
    if html != 0 {
        if let Some(raw) = clipboard.read::<u8>(html) {
            if let Ok(source) = std::str::from_utf8(nul_prefix(&raw)) {
                if let Some(converted) = neokey_win::data::convert_clipboard_html(source, options) {
                    entries.push((
                        html,
                        Block::new(converted.as_bytes()).ok_or("Không lưu được HTML.")?,
                    ));
                }
            }
        }
    }
    if entries.is_empty() {
        return Err("Clipboard không có văn bản để chuyển mã.".into());
    }
    clipboard
        .replace(&entries)
        .then_some(())
        .ok_or_else(|| "Không ghi được Clipboard.".into())
}
