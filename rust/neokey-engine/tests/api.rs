use neokey_engine::{
    data::*, vietnamese::character_to_key, ConvertOptions, Engine, Event, EventState, Settings,
    SmartSwitch,
};
fn type_text(engine: &mut Engine, text: &str) -> String {
    let mut screen = Vec::new();
    for c in text.chars() {
        let key = character_to_key(c).unwrap();
        let h = engine
            .handle_event(
                Event::Keyboard,
                EventState::KeyDown,
                key as u16,
                u8::from(key & CAPS_MASK as u32 != 0),
                false,
            )
            .clone();
        if [1, 3, 4, 5].contains(&h.code) {
            for _ in 0..h.backspace_count {
                screen.pop();
            }
            let chars: Vec<_> = if h.code == 4 {
                h.macro_data.clone()
            } else {
                h.char_data[..h.new_char_count as usize]
                    .iter()
                    .rev()
                    .copied()
                    .collect()
            };
            for data in chars {
                let code = engine.get_character_code(data);
                let c = if code & CHAR_CODE_MASK as u32 != 0 {
                    code as u16
                } else {
                    neokey_engine::vietnamese::key_code_to_character(code)
                };
                screen.push(c);
            }
            if h.code != 1 {
                screen.push(c as u16);
            }
        } else if c == '\u{8}' {
            screen.pop();
        } else {
            screen.push(c as u16);
        }
        if h.code == 5 {
            engine.start_new_session();
        }
    }
    String::from_utf16(&screen).unwrap()
}
#[test]
fn vietnamese_sentence_and_instance_isolation() {
    let mut a = Engine::new();
    let mut b = Engine::new();
    assert_eq!(type_text(&mut a, "Tieengs Vieetj"), "Tiếng Việt");
    assert_eq!(type_text(&mut b, "as"), "á");
    assert_eq!(
        type_text(
            &mut Engine::with_settings(Settings {
                vInputType: 1,
                ..Settings::default()
            }),
            "Tie6ng1 Vie6t5"
        ),
        "Tiếng Việt"
    );
}
#[test]
fn macro_lookup_persistence_and_independent_engines() {
    let mut a = Engine::with_settings(Settings {
        vUseMacro: 1,
        ..Settings::default()
    });
    let b = Engine::new();
    a.add_macro("vn", "Việt Nam");
    assert!(a.macros().has("vn", 0));
    assert!(!b.macros().has("vn", 0));
    assert_eq!(type_text(&mut a, "vn "), "Việt Nam ");
    let bytes = a.macros().save_data();
    a.init_macros(&bytes).unwrap();
    assert_eq!(bytes, a.macros().save_data());
    assert!(a.delete_macro("vn"));
    assert!(a.init_macros(&[1, 0, 255]).is_err());
}
#[test]
fn smart_switch_keeps_language_and_rejects_truncated_records() {
    let mut s = SmartSwitch::default();
    assert_eq!(s.get("org.editor", 1), -1);
    assert_eq!(s.get("org.editor", 0), 1);
    s.set("org.editor", 0);
    let bytes = s.save_data();
    let mut t = SmartSwitch::default();
    t.init(&bytes).unwrap();
    assert_eq!(t.get("org.editor", 1), 0);
    assert!(t.init(&[1, 0, 20]).is_err());
}

#[test]
fn smart_switch_iter_lists_entries_and_packed_helpers_round_trip() {
    let mut s = SmartSwitch::default();
    s.set("com.a", SmartSwitch::packed(1, 2));
    s.set("com.b", SmartSwitch::packed(0, 0));
    let listed: Vec<_> = s.iter().collect();
    assert_eq!(listed, vec![("com.a", 5), ("com.b", 0)]);
    assert_eq!(SmartSwitch::language(5), 1);
    assert_eq!(SmartSwitch::code_table(5), 2);
}
#[test]
fn convert_tool_preserves_upstream_default_lowercase_behavior() {
    assert_eq!(
        ConvertOptions::default().convert("ABC Tiếng Việt").unwrap(),
        "abc tiếng việt"
    );
    assert_eq!(
        ConvertOptions {
            to_all_caps: true,
            ..ConvertOptions::default()
        }
        .convert("tiếng Việt")
        .unwrap(),
        "TIẾNG VIỆT"
    );
}

#[test]
fn macro_encoding_stops_at_nul_and_preserves_serialized_text() {
    let mut e = Engine::with_settings(Settings {
        vUseMacro: 1,
        ..Settings::default()
    });
    let bytes = [1, 0, 3, b'a', 0, b'b', 3, 0, b'x', 0, b'y'];
    e.init_macros(&bytes).unwrap();
    assert_eq!(e.macros().save_data(), bytes);
    assert_eq!(type_text(&mut e, "a "), "x ");
}
