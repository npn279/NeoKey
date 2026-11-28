use std::{env, fs, path::PathBuf};

const ICONS: &[&str] = &[
    "icon.ico",
    "StatusViet.ico",
    "StatusEng.ico",
    "StatusViet10.ico",
    "StatusEng10.ico",
];

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let source = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("assets");
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let msvc = env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc");
    let mut resources = Vec::new();
    let mut image_id = 1;
    // A .res stream starts with a null resource header.
    resource(&mut resources, 0, 0, &[]);
    for (index, name) in ICONS.iter().enumerate() {
        let path = source.join(name);
        println!("cargo:rerun-if-changed={}", path.display());
        let icon = fs::read(path).expect("read the original OpenKey icon");
        if msvc {
            let count = u16::from_le_bytes([icon[4], icon[5]]);
            let mut group = icon[..6].to_vec();
            for entry in icon[6..6 + usize::from(count) * 16].as_chunks::<16>().0 {
                let size = u32::from_le_bytes(entry[8..12].try_into().unwrap()) as usize;
                let offset = u32::from_le_bytes(entry[12..16].try_into().unwrap()) as usize;
                resource(&mut resources, 3, image_id, &icon[offset..offset + size]);
                group.extend_from_slice(&entry[..12]);
                group.extend_from_slice(&image_id.to_le_bytes());
                image_id += 1;
            }
            resource(&mut resources, 14, 101 + index as u16, &group);
        } else {
            fs::write(output.join(name), icon).expect("copy the original OpenKey icon");
        }
    }
    if msvc {
        let path = output.join("neokey-icons.res");
        fs::write(&path, resources).expect("write Win32 icon resources");
        println!("cargo:rustc-link-arg-bin=neokey-win={}", path.display());
    }
}

// Win32 .res record with ordinal type/name and DWORD-aligned data.
fn resource(output: &mut Vec<u8>, kind: u16, id: u16, data: &[u8]) {
    output.extend_from_slice(&(data.len() as u32).to_le_bytes());
    output.extend_from_slice(&32u32.to_le_bytes());
    for word in [0xffff, kind, 0xffff, id] {
        output.extend_from_slice(&word.to_le_bytes());
    }
    // Data version, memory flags, language, version, characteristics.
    output.extend_from_slice(&[0; 16]);
    output.extend_from_slice(data);
    output.resize(output.len().next_multiple_of(4), 0);
}
