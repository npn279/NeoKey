//! Data passed between the Windows UI and engine.
use neokey_engine::{macros::Macros, Engine};
use std::{fs::File, io::Read, path::Path};

const MAX_MACRO_FILE: u64 = 16 * 1024 * 1024;

pub fn macro_fits(name: &str, content: &str) -> bool {
    !content.is_empty() && macro_fields_fit(name, content)
}

fn macro_fields_fit(name: &str, content: &str) -> bool {
    !name.is_empty()
        && name.len() <= u8::MAX as usize
        && content.len() <= u16::MAX as usize
        && !name.contains('\0')
        && !content.contains('\0')
}

pub fn checked_macros(engine: &Engine, path: &Path, append: bool) -> Result<Macros, String> {
    let mut text = String::new();
    File::open(path)
        .and_then(|file| file.take(MAX_MACRO_FILE + 1).read_to_string(&mut text))
        .map_err(|error| error.to_string())?;
    if text.len() as u64 > MAX_MACRO_FILE {
        return Err("Tệp gõ tắt quá lớn.".into());
    }
    import_macros(engine, &text, append)
}

fn import_macros(engine: &Engine, text: &str, append: bool) -> Result<Macros, String> {
    let mut macros = if append {
        engine.macros().clone()
    } else {
        Macros::default()
    };
    let table = engine.settings().vCodeTable as usize;
    // OpenKey/UniKey text files start with a header and use the first colon after
    // the first character so a shortcut may itself begin with a colon.
    for line in text.lines().skip(1) {
        let first = usize::from(line.starts_with(':'));
        let Some(separator) = line[first..].find(':').map(|index| first + index) else {
            continue;
        };
        let name = &line[..separator];
        let content = &line[separator + 1..];
        if !macros.has(name, table) {
            if !macro_fields_fit(name, content) || macros.all().size_hint().0 >= u16::MAX as usize {
                return Err("Từ gõ tắt hoặc nội dung vượt giới hạn lưu trữ.".into());
            }
            macros.add(name, content, table);
        }
    }
    Ok(macros)
}

pub const UPDATE_URL: &str = "https://raw.githubusercontent.com/npn279/NeoKey/main/version.json";
pub const RELEASES_URL: &str = "https://github.com/npn279/NeoKey/releases";

pub fn current_version_code() -> u32 {
    let component = |value: &str| {
        value
            .parse::<u32>()
            .expect("numeric Cargo version component")
    };
    component(env!("CARGO_PKG_VERSION_MAJOR"))
        | component(env!("CARGO_PKG_VERSION_MINOR")) << 8
        | component(env!("CARGO_PKG_VERSION_PATCH")) << 16
}

pub fn available_version(json: &[u8], local: u32) -> Result<Option<String>, &'static str> {
    let document: serde_json::Value =
        serde_json::from_slice(json).map_err(|_| "JSON không hợp lệ.")?;
    let windows = &document["latestWinVersion"];
    let code = windows["versionCode"]
        .as_u64()
        .filter(|code| *code <= 0xff_ffff)
        .ok_or("Thiếu mã phiên bản Windows.")? as u32;
    let name = windows["versionName"]
        .as_str()
        .filter(|name| !name.is_empty() && name.len() <= 64)
        .ok_or("Thiếu tên phiên bản Windows.")?;
    let order = |code: u32| (code & 0xff, (code >> 8) & 0xff, code >> 16);
    Ok((order(code) > order(local)).then(|| name.to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn existing_registry_macros_round_trip_through_the_engine() {
        let binary = [1, 0, 2, b'v', b'n', 3, 0, b'V', b'N', b'!'];
        let mut engine = Engine::new();
        engine.init_macros(&binary).unwrap();
        assert_eq!(engine.macros().all().next().unwrap().1.content, "VN!");
        assert_eq!(engine.macros().save_data(), binary);
        assert!(macro_fits("vn", "Việt Nam"));
        assert!(!macro_fits(&"ấ".repeat(86), "VN"));
    }

    #[test]
    fn imports_windows_macro_files_without_adding_a_carriage_return() {
        let mut engine = Engine::new();
        engine.add_macro("existing", "unchanged");
        let macros = import_macros(
            &engine,
            "; OpenKey
vn:Việt Nam
:vn:Việt
",
            true,
        )
        .unwrap();
        let entries: std::collections::BTreeMap<_, _> = macros
            .all()
            .map(|(_, entry)| (entry.text.as_str(), entry.content.as_str()))
            .collect();
        assert_eq!(entries["vn"], "Việt Nam");
        assert_eq!(entries[":vn"], "Việt");
        assert_eq!(entries["existing"], "unchanged");
    }

    #[test]
    fn conversion_recalculates_html_byte_offsets_without_rewriting_markup() {
        use neokey_engine::ConvertOptions;
        let snippet = "<b title='A>B'>á</b>&amp;";
        let html = |begin: usize, end, fragment_begin, fragment_end| {
            format!(
                "Version:1.0
StartHTML:{begin:04}
EndHTML:{end:04}
StartFragment:{fragment_begin:04}
EndFragment:{fragment_end:04}
"
            )
        };
        let start = html(0, 0, 0, 0).len();
        let source = html(start, start + snippet.len(), start, start + snippet.len()) + snippet;
        let options = ConvertOptions {
            remove_mark: true,
            ..ConvertOptions::default()
        };
        let converted = convert_clipboard_html(&source, &options).unwrap();
        let offset = |prefix: &str| {
            converted
                .lines()
                .find_map(|line| line.strip_prefix(prefix))
                .unwrap()
                .parse::<usize>()
                .unwrap()
        };
        assert_eq!(
            &converted[offset("StartFragment:")..offset("EndFragment:")],
            "<b title='A>B'>a</b>&amp;"
        );
        assert_eq!(offset("EndHTML:"), converted.len());
    }

    #[test]
    fn update_compares_windows_components_in_major_minor_patch_order() {
        let document = br#"{"latestVersion":{"versionCode":99},"latestWinVersion":{"versionCode":3,"versionName":"3.0.0"}}"#;
        assert_eq!(
            available_version(document, 0x05_00_02),
            Ok(Some("3.0.0".into()))
        );
        assert_eq!(available_version(document, 3), Ok(None));
        assert!(available_version(b"{}", 0).is_err());
    }
}

/// Convert the CF_HTML fragment while preserving its tags, entities and byte offsets.
pub fn convert_clipboard_html(
    source: &str,
    options: &neokey_engine::ConvertOptions,
) -> Option<String> {
    let offset = |name: &str| {
        source
            .lines()
            .take(24)
            .find_map(|line| line.strip_prefix(name))?
            .trim()
            .parse::<usize>()
            .ok()
    };
    let html_start = offset("StartHTML:");
    let html_end = offset("EndHTML:");
    let start = offset("StartFragment:")?;
    let end = offset("EndFragment:")?;
    let fragment = source.get(start..end)?;
    let prefix = source.get(html_start.unwrap_or(start)..start)?;
    let suffix = source.get(end..html_end.unwrap_or(end))?;
    let mut converted = String::new();
    let mut remaining = fragment;
    while !remaining.is_empty() {
        let text_end = remaining.find(['<', '&']).unwrap_or(remaining.len());
        converted.push_str(&options.convert(&remaining[..text_end]).ok()?);
        remaining = &remaining[text_end..];
        if remaining.is_empty() {
            break;
        }
        let mut end = 1;
        if remaining.starts_with("<!--") {
            end = remaining.find("-->")? + 3;
        } else if remaining.starts_with('<') {
            let mut quote = None;
            end = remaining.char_indices().skip(1).find_map(|(index, ch)| {
                match (quote, ch) {
                    (Some(open), ch) if ch == open => quote = None,
                    (None, '"' | '\'') => quote = Some(ch),
                    (None, '>') => return Some(index + 1),
                    _ => {}
                }
                None
            })?;
        } else if let Some(semi) = remaining
            .as_bytes()
            .iter()
            .take(34)
            .position(|&byte| byte == b';')
        {
            let entity = &remaining[1..semi];
            if !entity.is_empty()
                && entity
                    .bytes()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == b'#')
            {
                end = semi + 1;
            }
        }
        converted.push_str(&remaining[..end]);
        remaining = &remaining[end..];
    }

    let header = |begin, end, fragment_begin, fragment_end| {
        format!("Version:1.0\r\nStartHTML:{begin:010}\r\nEndHTML:{end:010}\r\nStartFragment:{fragment_begin:010}\r\nEndFragment:{fragment_end:010}\r\n")
    };
    let begin = header(0, 0, 0, 0).len();
    let fragment_begin = begin + prefix.len();
    let fragment_end = fragment_begin + converted.len();
    let end = fragment_end + suffix.len();
    Some(header(begin, end, fragment_begin, fragment_end) + prefix + &converted + suffix)
}
