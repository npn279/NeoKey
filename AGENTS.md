# AGENTS.md

This file provides guidance to coding agents (Codex, opencode, Claude Code) when working with code in this repository.

## What this is

NeoKey is a GPL-3.0 fork of [tuyenvm/OpenKey](https://github.com/tuyenvm/OpenKey), a Vietnamese input method (Telex / VNI / Simple Telex) that works with the "backspace technique": it intercepts keystrokes, then sends synthetic Backspaces plus the replacement characters, so nothing gets underlined. Everything is written in **Rust** under `rust/`. The original Objective-C macOS app and Win32 C++ app were removed. `FIXES.md` lists what the fork changed (event-tap lag and Spotlight/Alfred/Raycast diacritics). Docs and commit history are mostly in Vietnamese.

## Build and test

From `rust/` (Cargo workspace; on this Mac, `export PATH=/opt/homebrew/opt/rustup/bin:$PATH`):

```bash
cargo test --workspace
cargo run --release -p engine-diff     # Rust engine vs original C++ engine, must report 0 mismatches
./test-c-abi.sh                        # C caller against the Rust static library
./neokey-mac/bundle.sh                 # builds and ad-hoc signs target/NeoKey.app
cargo check -p neokey-win --target x86_64-pc-windows-msvc   # Windows app; cannot link on macOS
```

The macOS app needs Accessibility permission, and any other Vietnamese input method should be turned off while testing. The Windows binary can only be linked and run on Windows. CI (`.github/workflows/rust.yml`) tests and bundles on `macos-latest` and builds `neokey-win.exe` on `windows-latest`.

## Architecture

- `rust/neokey-engine`: the typing engine, a faithful port of the original C++ (`core/` state machines, `vietnamese.rs` code tables, `macros.rs`, `smart_switch.rs`, `convert.rs`, `settings.rs` for every `v…` setting). Engine key codes are **macOS virtual key codes** (`KEY_A` = 0). `ffi.rs` + `include/neokey_engine.h` expose a C ABI.
- `rust/engine-diff`: differential oracle. It compiles the **unchanged original C++ engine** in `Sources/OpenKey/engine/` (kept only for this) and compares every hook field and the simulated screen against the Rust engine. Keep engine behaviour identical unless a change is deliberate and the oracle is updated.
- `rust/neokey-mac`: macOS menu bar app (`objc2`). `typing.rs` is the platform-independent `Typing::process_key` (unit-tested); `macos/event_tap.rs` and `macos/mod.rs` run the `CGEventTap`; `macos/accessibility.rs` does the AX rewrite; `macos/ui/` is the control panel and settings window. Settings live in the `com.neokey.rust` defaults domain and are imported once from the old `com.tuyenmai.openkey` domain.
- `rust/neokey-win`: Windows tray app. `typing.rs` translates Windows VK codes to engine (mac) codes (`to_engine`) and is tested on every OS; `win/` holds the low-level hooks, `SendInput`, tray, dialogs and registry code (`cfg(windows)`). Settings use the original registry keys under `HKCU\SOFTWARE\TuyenMai\OpenKey`, so OpenKey users keep theirs.

### Engine ↔ platform contract

- Each `Engine` owns its `Settings`. For each key the platform calls `handle_event(...)` and reads the returned `HookState`: `code` (`vDoNothing` / `vWillProcess` / `vBreakWord` / `vRestore` / `vReplaceMaro` / …), `backspace_count`, `new_char_count`, `char_data` (**reverse order**, decoded with `get_character_code`), `ext_code`, and `macro_data` (forward order). The platform deletes `backspace_count` characters and types the new ones.

### macOS event flow

- The tap callback must stay fast: anything slow makes macOS disable the tap (`kCGEventTapDisabledByTimeout`) and typing silently stops. The callback re-enables the tap on timeout or user-input disable, and a 1-second watchdog checks `CGEventTapIsEnabled`. **Keep heavy IPC out of the per-key path.** The front app's bundle id comes from the event's target PID and is cached per PID.
- Spotlight, Alfred (`com.runningwithcrayons.Alfred`, `-3`) and Raycast (`com.raycast.macos`) drop synthetic Backspaces. For these the app first rewrites the focused field through the Accessibility API (UTF-16, 80 ms messaging timeout). If that fails, it selects the characters with Shift+Left and types over them.

## Versioning

Bump together: `version.json` at the repo root (`latestVersion` for macOS, `latestWinVersion` for Windows), `rust/neokey-mac/Info.plist` (`CFBundleShortVersionString` / `CFBundleVersion` = `versionCode`), and `version` in `rust/neokey-win/Cargo.toml` (encoded as `major | minor << 8 | patch << 16` for `versionCode`). Both update checkers fetch `version.json` from `npn279/NeoKey` on `main`. Record changes in `CHANGELOG.md`.
