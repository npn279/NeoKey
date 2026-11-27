# NeoKey Rust

A GPL-3.0 port of `Sources/OpenKey/engine`, using the **macOS virtual key codes** from
`platforms/mac.h`. The engine library has no C++ or third-party dependencies. This workspace also
includes a native macOS menu bar app that uses the Rust engine.

## Layout

- `neokey-engine`: `rlib` and `staticlib`, safe independent `Engine` instances.
  - `src/core/`: the original composition, spelling, mark placement, history and event
    state machines, partitioned by responsibility. Upstream function/scratch-variable names
    and integer booleans are retained to make auditing the port straightforward. Explicit
    unsigned narrowing preserves C++ byte/short semantics, including VNI with caps.
  - `src/data.rs`, `src/vietnamese.rs`: all original macOS keys, packed flags, vowel/consonant
    tables and five code tables (Unicode, TCVN3, VNI-Windows, compound Unicode, CP1258).
  - `src/macros.rs`, `src/smart_switch.rs`, `src/convert.rs`: macro lookup/import/export,
    per-app language memory and code-table conversion.
  - `src/settings.rs`: every `v…` setting declared in `Engine.h`. Settings used only by
    platform code remain available without adding platform behavior to the engine.
  - `src/ffi.rs`, `include/neokey_engine.h`: C/ObjC bridge. All unsafe Rust is confined to
    FFI modules; the rest of each crate denies unsafe code.
- `neokey-mac`: native menu bar app and a platform-independent `Typing::process_key`
  interface for tests and benchmarks.
- `engine-diff`: independent original C++ oracle, corpus generator, seeded random tests
  and executable. Only this crate depends on `cc`.
- `test-c-abi.sh`, `neokey-engine/tests/c_abi.c`: compile, link and execute a real C caller
  against the Rust static library.

## Run

From this directory, with Rust and a C++11 compiler installed:

```sh
export PATH=/opt/homebrew/opt/rustup/bin:$PATH
cargo test --workspace
cargo run --release -p engine-diff
./test-c-abi.sh
```

If the Cargo cache is read-only in a sandbox, also set
`export CARGO_HOME=/private/tmp/neokey-cargo-home` before these commands.
`cargo build --release -p neokey-engine` produces `target/release/libneokey_engine.a`.
The C smoke script shows the system libraries needed to link it.

## macOS app

Build and launch from `rust/` on a Mac:

```sh
export PATH=/opt/homebrew/opt/rustup/bin:$PATH
cargo build --release --workspace
cargo test --workspace
./neokey-mac/bundle.sh
open target/NeoKey.app
```

The script builds and ad-hoc signs `target/NeoKey.app`. Grant NeoKey access
under **System Settings → Privacy & Security → Accessibility** when prompted. If
you grant access while the app bundle is running, NeoKey relaunches once so
the new process can use Accessibility in Spotlight, Alfred and Raycast. The bare
binary enables typing after permission is granted and logs a recommendation to
restart it for launcher support. Disable other Vietnamese input methods while
using NeoKey so only one tool processes each key.

Left-click the **V** or **E** status item to switch language. Right-click for
Telex, VNI, Simple Telex 1/2, the five code tables, spell check and Quit. The
default hotkey is **Option+Z**. The app uses the existing typing preference key
names and binary macro and smart-switch formats in the separate
`com.neokey.rust` defaults domain. Set options outside the menu with `defaults`
while the app is quit, then relaunch it. For example:

```sh
defaults write com.neokey.rust SendKeyStepByStep -int 1
```

The bare Cargo executable uses its process defaults domain. Launch the app bundle
to use the `com.neokey.rust` preferences.

The event tap uses a bounded PID-to-bundle-ID cache and a one-second watchdog.
Spotlight, Alfred and Raycast get a UTF-16 Accessibility edit with an 80 ms
messaging timeout per AX request. When the field cannot be rewritten, the app
selects the composing text with Shift+Left and types its replacement.

The Rust app covers typing and the menu bar controls. The Objective-C app's
preferences window, macro editor, clipboard conversion tool and its shortcut,
login item, update checker, and onboarding screens are outside this app's scope.
Existing Objective-C preferences are in a separate domain, so settings and macros
start fresh unless copied into the Rust domain. Input-source language changes are
picked up by the one-second watchdog. Secure Input can prevent global taps from
receiving keystrokes.

For callback benchmarks, the library target exports `neokey_mac::typing`.
Feed platform virtual key codes, modifier flags and mouse events to
`Typing::process_key`. It returns the edit, pass-through, suppression or language
switch instruction without installing a tap or calling AppKit.

## Safe Rust API

```rust
use neokey_engine::{Engine, Event, EventState, Settings};
use neokey_engine::data::{KEY_A, KEY_S};

let mut engine = Engine::with_settings(Settings::default());
engine.handle_event(Event::Keyboard, EventState::KeyDown, KEY_A as u16, 0, false);
let hook = engine.handle_event(Event::Keyboard, EventState::KeyDown, KEY_S as u16, 0, false);
assert_eq!((hook.code, hook.backspace_count, hook.new_char_count), (1, 1, 1));
assert_eq!(hook.char_data[0] & 0xffff, 0x00e1); // á
```

`Engine::new()` uses default settings. Use `settings()`/`set_settings()` to configure
all settings; the latter refreshes the spelling baseline and macro output table. Input
modes are 0–3 and tables 0–4; invalid values are rejected with an assertion by the Rust
API and `false` by the C bridge. Other settings retain their original integer values.

`handle_event()` mirrors `vKeyHandleEvent`; select `english_mode()` explicitly as the
original platform does. Both return the current `HookState`, including stale fields
that C++ intentionally leaves untouched. Replacement `char_data` is **reverse ordered**;
`macro_data` is forward ordered. `get_character_code()` converts internal packed engine
characters to the selected code-table output. `vDoNothing` passes the physical key;
process/restore codes delete the indicated logical characters then insert replacement
codes; restore/macro codes additionally pass the triggering key. Compound/VNI output
can use multiple UTF-16 units per logical character.

`start_new_session()` mirrors the original partial word reset. `initialize()` mirrors
`vKeyInit()`'s history reset. Neither means constructing a completely fresh engine;
use `Engine::new()` for that. Temporary controls are `temp_off_spell_checking()`,
`set_check_spelling()` and `temp_off_engine(bool)`.

Macro methods include `add_macro`, `delete_macro`, `init_macros`, `find_macro`,
`read_macros_from_file`, and `on_table_code_change`; `macros()` exposes enumeration,
lookup by name, binary serialization and text export. `SmartSwitch` exposes `init`,
`save_data`, `get`, `set`, preserving upstream signed-byte storage and cache behavior.
`ConvertOptions::convert` implements all original conversion/case/mark options.
Persistence uses the original little-endian formats and ordered-map serialization.

## C ABI

Include `neokey-engine/include/neokey_engine.h`. The bridge exports `vKeyInit`,
`vKeyHandleEvent`, `startNewSession`, `getCharacterCode`, `vEnglishMode`, temporary
controls, macro functions, smart-switch functions and conversion. It adds settings
get/set/default functions instead of mutable C globals.

Each calling thread has its own legacy engine. Keep one composition on one thread.
The returned hook pointer remains stable until that thread exits. Copy the hook and
macro buffers before the next mutating call; serialization/conversion buffers last
until the next serialization/conversion. The header documents ownership and null input
rules. This is a C ABI with pointer/count fields for vectors, **not** the C++
`std::vector` layout or a drop-in replacement for existing mangled C++ symbols.

## Differential verification and achieved result

`engine-diff/build.rs` uses `cc` with C++11. `shim.cpp` includes the **unchanged original
Engine.cpp**, allowing test-only reset of its private static state to the same initial
state as a new Rust `Engine`. The other four original `.cpp` files compile normally.
The shim defines/configures the actual `v…` globals and copies the actual hook output.
No Rust engine code is called by the oracle, and no C++ is linked into `neokey-engine`.

For each event the harness compares code, backspace/new-character counts, extCode,
**all 32 charData slots**, macroKey and macroData. Independent Rust/C++ character
and key-code decoding feeds a screen model that applies backspaces and replacements;
screen text is checked after every event and at each sequence end. It models logical
characters so VNI/compound multi-unit output is deleted together. It does not exercise
macOS Accessibility/event injection or application-specific text handling.

The full run uses:

- Real Vietnamese sentences, names, tone/shape families and punctuation, converted
  programmatically into Telex, VNI, Simple Telex 1 and Simple Telex 2 keystrokes.
- All five code tables × 64 distinct profiles over 12 Boolean settings. Profiles include
  all-off/all-on and cover every value combination for each pair of settings, checked
  by a test. They cover spelling, orthography, quick Telex, free mark, spelling restore,
  macros, first-letter caps, quick start/end consonants, ZFWJ, macro caps and browser fix.
- 1,024 continuous random events per configuration (1,310,720 random events), using
  xorshift64 seed `0x4e656f4b6579`; letters, digits, caps/shift, backspace, spaces,
  punctuation, control-modified events, mouse events and key-up events. Dedicated
  sequences include long words, restoration, history editing and macros.
- Separate English-mode cases, temporary toggles, decoder probes, macro serialization
  (including supplementary Unicode and embedded NUL), smart-switch persistence, and
  800 conversions covering all 25 table pairs × 32 conversion-flag combinations.

**Verified on macOS arm64, 2026-11-27:**

```text
Compared 2645560 keystrokes; 0 mismatches; elapsed 7.656s
```

The number counts event comparisons in the sequence matrix and English cases; utility
assertions are additional and are not inflated into the keystroke total. This is a CPU
run with both implementations, complete hook comparisons and screen simulation; it is
not an engine-only throughput benchmark. `cargo test --workspace` runs a smaller
matrix plus focused API/regression tests. The C static-library smoke test also passes.

Upstream quirks are preserved, including default converter lowercasing, macro case
rules, embedded-NUL text encoding and smart-switch cache behavior. Malformed/truncated
persistence is rejected safely instead of reproducing C++ out-of-bounds reads. Conversion
returns `Result`; an invalid UTF-16 result corresponds to the original converter's
`std::range_error` (the C bridge returns null). Character decoding safely retains the
input for unsupported packed combinations rather than dereferencing outside a table.
As with any differential test, these runs establish equality on the exercised events,
not equivalence for C++ undefined behavior. Original fixed-buffer edge cases (for
example quick-start expansion of a full 32-character word advertising 33 output
characters) remain outside the valid output-buffer contract.
