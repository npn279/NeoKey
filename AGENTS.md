# AGENTS.md

This file provides guidance to coding agents (Codex, opencode, Claude Code) when working with code in this repository.

## What this is

NeoKey is a GPL-3.0 fork of [tuyenvm/OpenKey](https://github.com/tuyenvm/OpenKey), a Vietnamese input method (Telex / VNI / Simple Telex) that works with the "backspace technique": it intercepts keystrokes, then sends synthetic Backspaces plus the replacement characters, so nothing gets underlined. The fork's active work is the **macOS app**. `FIXES.md` lists what the fork changed (event-tap lag and Spotlight/Alfred/Raycast diacritics). Docs and commit history are mostly in Vietnamese.

## Build and test

macOS app (needs Xcode on a Mac; Linux cannot build it):

```bash
xcodebuild -project Sources/OpenKey/macOS/OpenKey.xcodeproj -scheme OpenKey -configuration Debug build
```

Schemes: `OpenKey` (main app), `OpenKeyHelper` (login-item launcher, enabled via `SMLoginItemSetEnabled` with `com.tuyenmai.OpenKeyHelper`), `OpenKeyUpdate`. For release builds, use Product → Archive → Distribute App in Xcode. The built app needs Accessibility permission, and any other Vietnamese input method should be turned off while testing. If Xcode reports `invalid in C99` in `MJAccessibilityUtils.m`, add `extern BOOL AXAPIEnabled();` at the top of the file.

Windows: `Sources/OpenKey/win32/OpenKey/OpenKey.sln` (MSBuild, x86 + x64). CI (`.github/workflows/msbuild.yml`) builds only this, and only on `master`, which this fork does not use (it uses `main`). So nothing in CI builds or tests the macOS code.

The only test is a standalone C file:

```bash
cc -std=c99 -Wall -o /tmp/composition_replace tests/composition_replace.c && /tmp/composition_replace
```

`tests/composition_replace.c` is a **hand-copied mirror** of `stringByReplacingComposition` in `macOS/ModernKey/OpenKey.mm`, working in UTF-16 code units. If you change one, change the other to match.

## Architecture

Code lives under `Sources/OpenKey/`:

- `engine/`: platform-independent C++ core shared by every OS. `Engine.cpp` (the typing state machine), `Vietnamese.cpp` (character/code-table data), `Macro.cpp` (text shortcuts), `SmartSwitchKey.cpp` (per-app language memory), `ConvertTool.cpp` (converts between code tables). `DataType.h` picks the key-code header from `engine/platforms/{mac,win32,linux}.h` based on `LINUX` / `_WIN32`, with mac as the default. Engine key codes are **platform virtual key codes** (`KEY_A` = 0 on mac).
- `macOS/ModernKey/`: the Cocoa app. `OpenKey.mm` is the Objective-C++ bridge between the OS and the engine. `AppDelegate.m`, `ViewController.m` and the other view controllers are the UI, and `OpenKeyManager.m` creates the event tap and checks for updates.
- `win32/`: the Win32 app, using the same engine. `linux/` is a placeholder.

### Engine ↔ platform contract

- The engine is configured through global `extern int v…` settings declared in `Engine.h` (`vLanguage`, `vInputType`, `vCodeTable`, `vCheckSpelling`, `vUseMacro`, …). The platform layer defines these globals and loads and saves them. On macOS they are stored in `NSUserDefaults` (for example `vLanguage` ↔ key `InputMethod`).
- `vKeyInit()` returns a pointer to the shared `vKeyHookState` (`DataType.h`). For each key, the platform calls `vKeyHandleEvent(...)`, then reads that struct: `code` (`vDoNothing` / `vWillProcess` / `vBreakWord` / `vRestore` / `vReplaceMaro` / …), `backspaceCount`, `newCharCount`, `charData[]` (stored in reverse order and decoded with `getCharacterCode`), `extCode`, and `macroData` for macros. The platform carries out the result: it deletes `backspaceCount` characters and types the new ones.

### macOS event flow (`OpenKey.mm`)

- `OpenKeyCallback` is a `CGEventTap` callback. Anything slow in it makes macOS disable the tap (`kCGEventTapDisabledByTimeout`), and typing silently stops. So the callback re-enables the tap on timeout or user-input disable, and a 1-second watchdog calls `CGEventTapIsEnabled`. **Keep heavy IPC out of the per-key path.** The front app's bundle id comes from `kCGEventTargetUnixProcessID` and is cached per PID. `NSWorkspace` and `CGWindowListCopyWindowInfo` are called only when the PID changes.
- Output is sent with `CGEventTapPostEvent` as synthetic Backspace and Unicode key events. The "send keys one by one" setting changes whether characters are batched or sent singly.
- Spotlight, Alfred (`com.runningwithcrayons.Alfred`, `-3`) and Raycast (`com.raycast.macos`) drop synthetic Backspaces. For these bundle ids the app skips Backspace. It first rewrites the focused field's value through the Accessibility API (`kAXValueAttribute` + `kAXSelectedTextRangeAttribute`, 80 ms messaging timeout). If that fails, it selects the characters with Shift+Left and types over them.

## Versioning

The version lives in two places, and both must be bumped together: `MARKETING_VERSION` / `CURRENT_PROJECT_VERSION` in `macOS/OpenKey.xcodeproj/project.pbxproj`, and `version.json` at the repo root. The app's update checker (`OpenKeyManager.m`) fetches `version.json` from `npn279/NeoKey` on `main` and compares `versionCode`. Record changes in `CHANGELOG.md`.
