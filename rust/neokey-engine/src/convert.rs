//! ConvertTool.cpp's five code tables and case/mark options.
use crate::vietnamese::*;
#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct ConvertOptions {
    pub dont_alert_when_completed: bool,
    pub to_all_caps: bool,
    pub to_all_non_caps: bool,
    pub to_caps_first_letter: bool,
    pub to_caps_each_word: bool,
    pub remove_mark: bool,
    pub from_code: u8,
    pub to_code: u8,
    pub hot_key: i32,
}
fn find(code: i64, table: usize) -> Option<(i64, usize)> {
    for (k, v) in _codeTable[table].iter() {
        if let Some(i) = v.iter().position(|c| *c == code) {
            return Some((*k, i));
        }
    }
    None
}
// C++ uses the default C locale's towupper/towlower: only ASCII letters change.
fn upper(c: u16) -> u16 {
    if c <= 127 {
        (c as u8).to_ascii_uppercase() as u16
    } else {
        c
    }
}
fn lower(c: u16) -> u16 {
    if c <= 127 {
        (c as u8).to_ascii_lowercase() as u16
    } else {
        c
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConversionError;
impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("conversion produced invalid UTF-16")
    }
}
impl std::error::Error for ConversionError {}
impl ConvertOptions {
    pub fn convert(&self, source: &str) -> Result<String, ConversionError> {
        assert!(self.from_code < 5 && self.to_code < 5);
        let data: Vec<u16> = source.encode_utf16().take_while(|c| *c != 0).collect();
        let mut out = Vec::new();
        let mut i = 0;
        let mut has_break = false;
        let mut should_upper =
            (self.to_caps_first_letter || self.to_caps_each_word) && !self.to_all_non_caps;
        while i < data.len() {
            let t = data[i];
            let mut found = None;
            let mut paired = false;
            if i + 1 < data.len() {
                let mut p = false;
                let code = match self.from_code {
                    2 | 4 => {
                        p = true;
                        t as i64 | ((data[i + 1] as i64) << 8)
                    }
                    3 => {
                        if let Some(n) = _unicodeCompoundMark
                            .iter()
                            .position(|c| *c == data[i + 1] as i64)
                        {
                            p = true;
                            t as i64 | (((n + 1) as i64) << 13)
                        } else {
                            t as i64
                        }
                    }
                    _ => t as i64,
                } & 0xffff;
                found = find(code, self.from_code as usize);
                paired = found.is_some();
                if found.is_some() && p {
                    i += 1;
                }
            }
            if found.is_none() {
                found = find(t as i64, self.from_code as usize);
            }
            if let Some((key, mut k)) = found {
                if (self.to_all_caps || should_upper) && k % 2 != 0 {
                    k -= 1;
                } else if (self.to_all_non_caps || !should_upper) && k % 2 == 0 {
                    k += 1;
                }
                let mut target = _codeTable[self.to_code as usize][&key][k] as u16;
                if self.remove_mark {
                    target = key_code_to_character(key as u8 as u32);
                    if self.to_all_caps {
                        target = upper(target);
                    } else if self.to_all_non_caps {
                        target = lower(target);
                    }
                }
                if paired {
                    match self.to_code {
                        2 | 4 => {
                            out.push(target & 255);
                            if target >> 8 > 32 {
                                out.push(target >> 8);
                            }
                        }
                        3 => {
                            if target >> 13 > 0 {
                                out.push(target & 0x1fff);
                                out.push(
                                    _unicodeCompoundMark[((target >> 13) - 1) as usize] as u16,
                                );
                            } else {
                                out.push(target);
                            }
                        }
                        _ => out.push(target),
                    }
                } else {
                    out.push(target);
                }
                should_upper = false;
                has_break = false;
                i += 1;
                continue;
            }
            out.push(if self.to_all_caps || should_upper {
                upper(t)
            } else {
                lower(t)
            });
            if t == 10 || (has_break && t == 32) {
                if self.to_caps_first_letter || self.to_caps_each_word {
                    should_upper = true;
                }
            } else if t == 32 && self.to_caps_each_word {
                should_upper = true;
            } else if [46, 63, 33].contains(&t) {
                has_break = true;
            } else {
                should_upper = false;
                has_break = false;
            }
            i += 1;
        }
        out.truncate(out.iter().position(|c| *c == 0).unwrap_or(out.len()));
        String::from_utf16(&out).map_err(|_| ConversionError)
    }
}
