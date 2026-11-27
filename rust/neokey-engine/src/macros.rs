//! OpenKey-compatible macro lookup and persistence.
use crate::{data::*, vietnamese::*};
use std::{collections::BTreeMap, io, path::Path};
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidData;
impl std::fmt::Display for InvalidData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("truncated or invalid OpenKey data")
    }
}
impl std::error::Error for InvalidData {}
#[derive(Debug, Clone)]
pub struct MacroData {
    pub text: String,
    pub content: String,
    pub content_code: Vec<u32>,
}
#[derive(Debug, Clone, Default)]
pub struct Macros {
    entries: BTreeMap<Vec<i64>, MacroData>,
}
pub(crate) fn encode(text: &str, table: usize) -> Vec<i64> {
    text.encode_utf16()
        .take_while(|c| *c != 0)
        .map(|t| {
            if let Some(k) = _characterMap.get(&(t as i64)) {
                return *k;
            }
            for (k, v) in _codeTable[0].iter() {
                if let Some(i) = v.iter().position(|v| *v == t as i64) {
                    return _codeTable[table][k][i] | CHAR_CODE_MASK;
                }
            }
            t as i64 | PURE_CHARACTER_MASK
        })
        .collect()
}
fn modify_case(code: &mut i64, upper: bool, table: usize) -> bool {
    let old = *code;
    if *code & CHAR_CODE_MASK == 0 {
        // Preserve upstream's &= CAPS_MASK behavior (including its surprising uppercase result).
        *code &= if upper { CAPS_MASK } else { !CAPS_MASK };
        return *code != old;
    }
    for v in _codeTable[table].values() {
        if let Some(mut i) = v.iter().position(|v| *v == (*code & CHAR_MASK)) {
            if i % 2 == 0 && !upper {
                i += 1;
            } else if i % 2 != 0 && upper {
                i -= 1;
            }
            *code = v[i] | CHAR_CODE_MASK;
            return *code != old;
        }
    }
    false
}
impl Macros {
    pub fn add(&mut self, text: &str, content: &str, table: usize) -> bool {
        let k = encode(text, table);
        if let Some(entry) = self.entries.get_mut(&k) {
            entry.content = content.into();
            entry.content_code = encode(content, table)
                .into_iter()
                .map(|v| v as u32)
                .collect();
        } else {
            self.entries.insert(
                k,
                MacroData {
                    text: text.into(),
                    content: content.into(),
                    content_code: encode(content, table)
                        .into_iter()
                        .map(|v| v as u32)
                        .collect(),
                },
            );
        }
        true
    }
    pub fn has(&self, text: &str, table: usize) -> bool {
        self.entries.contains_key(&encode(text, table))
    }
    pub fn delete(&mut self, text: &str, table: usize) -> bool {
        self.entries.remove(&encode(text, table)).is_some()
    }
    pub fn all(&self) -> impl Iterator<Item = (&[i64], &MacroData)> {
        self.entries.iter().map(|(k, v)| (k.as_slice(), v))
    }
    pub fn on_table_code_change(&mut self, table: usize) {
        for v in self.entries.values_mut() {
            v.content_code = encode(&v.content, table)
                .into_iter()
                .map(|v| v as u32)
                .collect();
        }
    }
    pub(crate) fn find(&self, key: &mut [i64], table: usize, auto_caps: bool) -> Option<Vec<i64>> {
        if let Some(v) = self.entries.get(key) {
            return Some(v.content_code.iter().map(|v| *v as i64).collect());
        }
        if !auto_caps {
            return None;
        }
        let mut all_caps = false;
        if key.len() > 1 && modify_case(&mut key[1], false, table) {
            all_caps = true;
            for c in &mut key[2..] {
                modify_case(c, false, table);
            }
        }
        if !key.is_empty() && modify_case(&mut key[0], false, table) {
            if let Some(v) = self.entries.get(key) {
                let mut data: Vec<i64> = v.content_code.iter().map(|v| *v as i64).collect();
                for (i, c) in data.iter_mut().enumerate() {
                    if i == 0 || all_caps {
                        let ch = key_code_to_character(*c as u32);
                        if ch != 0 {
                            let ch = if ch <= 127 {
                                (ch as u8).to_ascii_uppercase() as i64
                            } else {
                                ch as i64
                            };
                            *c = *_characterMap.get(&ch).unwrap_or(&0);
                            continue;
                        }
                        if *c & CHAR_CODE_MASK != 0 {
                            modify_case(c, true, table);
                        }
                    }
                }
                return Some(data);
            }
        }
        None
    }
    pub fn init(&mut self, data: &[u8], table: usize) -> Result<(), InvalidData> {
        self.entries.clear();
        if data.len() < 2 {
            return Ok(());
        }
        let mut cursor = 2;
        for _ in 0..u16::from_le_bytes([data[0], data[1]]) {
            let n = *data.get(cursor).ok_or(InvalidData)? as usize;
            cursor += 1;
            let text = take_string(data, &mut cursor, n)?;
            let b = data.get(cursor..cursor + 2).ok_or(InvalidData)?;
            let n = u16::from_le_bytes([b[0], b[1]]) as usize;
            cursor += 2;
            let content = take_string(data, &mut cursor, n)?;
            self.entries.insert(
                encode(&text, table),
                MacroData {
                    text,
                    content_code: encode(&content, table)
                        .into_iter()
                        .map(|v| v as u32)
                        .collect(),
                    content,
                },
            );
        }
        Ok(())
    }
    /// Same little-endian byte format and BTreeMap ordering as getMacroSaveData.
    pub fn save_data(&self) -> Vec<u8> {
        let mut out = (self.entries.len() as u16).to_le_bytes().to_vec();
        for v in self.entries.values() {
            out.push(v.text.len() as u8);
            out.extend(v.text.as_bytes());
            let n = v.content.len() as u16;
            out.extend(n.to_le_bytes());
            out.extend(&v.content.as_bytes()[..n as usize]);
        }
        out
    }
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let mut text =
            String::from(";Compatible OpenKey Macro Data file for UniKey*** version=1 ***\n");
        for v in self.entries.values() {
            text.push_str(&format!("{}:{}\n", v.text, v.content));
        }
        std::fs::write(path, text)
    }
    pub fn read_from_file(
        &mut self,
        path: impl AsRef<Path>,
        append: bool,
        table: usize,
    ) -> io::Result<()> {
        let text = std::fs::read_to_string(path)?;
        if !append {
            self.entries.clear();
        }
        for line in text.split_terminator('\n').skip(1) {
            if let Some((mut name, mut content)) = line.split_once(':') {
                let mut owned = String::new();
                if name.is_empty() && !content.is_empty() {
                    if let Some((a, b)) = content.split_once(':') {
                        owned.push(':');
                        owned.push_str(a);
                        name = &owned;
                        content = b;
                    }
                }
                if !name.is_empty() && !self.has(name, table) {
                    self.add(name, content, table);
                }
            }
        }
        Ok(())
    }
}
pub(crate) fn take_string(
    data: &[u8],
    cursor: &mut usize,
    n: usize,
) -> Result<String, InvalidData> {
    let s = data.get(*cursor..*cursor + n).ok_or(InvalidData)?;
    *cursor += n;
    String::from_utf8(s.to_vec()).map_err(|_| InvalidData)
}
