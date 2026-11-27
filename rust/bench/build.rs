fn main() {
    let mac = "../../Sources/OpenKey/macOS/ModernKey";
    let engine = "../../Sources/OpenKey/engine";
    let mut build = cc::Build::new();
    build
        .cpp(true)
        .std("c++11")
        .warnings(false)
        .flag("-fobjc-arc")
        .include(mac)
        .include(engine)
        .file("callback.mm");
    if std::env::var("PROFILE").as_deref() == Ok("release") {
        build.opt_level(3).flag("-flto=full");
    }
    build.compile("openkey_callback");
    for framework in ["AppKit", "ApplicationServices", "Carbon", "CoreGraphics"] {
        println!("cargo:rustc-link-lib=framework={framework}");
    }
    for path in ["callback.mm", mac, engine] {
        println!("cargo:rerun-if-changed={path}");
    }
}
