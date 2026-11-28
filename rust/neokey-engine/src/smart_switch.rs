//! Per-application language/code-table memory, including the original cache semantics.
use crate::macros::{take_string, InvalidData};
use std::collections::BTreeMap;
#[derive(Clone, Debug, Default)]
pub struct SmartSwitch {
    entries: BTreeMap<String, i8>,
    cache_key: String,
    cache_data: i8,
}
impl SmartSwitch {
    pub fn init(&mut self, data: &[u8]) -> Result<(), InvalidData> {
        self.entries.clear();
        if data.len() < 2 {
            return Ok(());
        }
        let mut p = 2;
        for _ in 0..u16::from_le_bytes([data[0], data[1]]) {
            let n = *data.get(p).ok_or(InvalidData)? as usize;
            p += 1;
            let key = take_string(data, &mut p, n)?;
            let value = *data.get(p).ok_or(InvalidData)? as i8;
            p += 1;
            self.entries.insert(key, value);
        }
        Ok(())
    }
    pub fn save_data(&self) -> Vec<u8> {
        let mut out = (self.entries.len() as u16).to_le_bytes().to_vec();
        for (k, v) in &self.entries {
            out.push(k.len() as u8);
            out.extend(k.as_bytes());
            out.push(*v as u8);
        }
        out
    }
    pub fn get(&mut self, bundle_id: &str, current: i32) -> i32 {
        if self.cache_key == bundle_id {
            return self.cache_data as i32;
        }
        self.cache_key = bundle_id.into();
        if let Some(v) = self.entries.get(bundle_id) {
            self.cache_data = *v;
            return *v as i32;
        }
        self.cache_data = current as i8;
        self.entries.insert(bundle_id.into(), current as i8);
        -1
    }
    pub fn set(&mut self, bundle_id: &str, language: i32) {
        self.entries.insert(bundle_id.into(), language as i8);
        self.cache_key = bundle_id.into();
        self.cache_data = language as i8;
    }

    /// Read-only view of stored per-app values, ordered by bundle id.
    pub fn iter(&self) -> impl Iterator<Item = (&str, i8)> + '_ {
        self.entries.iter().map(|(k, v)| (k.as_str(), *v))
    }

    pub fn language(stored: i8) -> i64 {
        i64::from(stored & 1)
    }

    pub fn code_table(stored: i8) -> i64 {
        i64::from(stored >> 1)
    }

    pub fn packed(language: i64, code_table: i64) -> i32 {
        (language | (code_table << 1)) as i32
    }
}
