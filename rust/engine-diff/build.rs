fn main() {
    let base = "../../Sources/OpenKey/engine";
    let mut build = cc::Build::new();
    build
        .warnings(false)
        .cpp(true)
        .std("c++11")
        .include(base)
        .file("shim.cpp");
    // shim.cpp includes Engine.cpp unchanged so each independent trial can reset its file-static state.
    for name in ["Vietnamese", "Macro", "SmartSwitchKey", "ConvertTool"] {
        build.file(format!("{base}/{name}.cpp"));
    }
    // Match the Rust workspace Release profile for benchmarks and differential runs.
    if std::env::var("PROFILE").as_deref() == Ok("release")
        && std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos")
    {
        build.opt_level(3).flag("-flto=full");
    }
    build
        .flag_if_supported("-Wno-unused-value")
        .compile("openkey_reference");
    println!("cargo:rerun-if-changed=shim.cpp");
    println!("cargo:rerun-if-changed={base}");
}
