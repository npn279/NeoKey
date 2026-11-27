# NeoKey efficiency measurements

Measured 2026-11-27T22:27:00+07:00 on **Mac16,12**, Apple M4,
10 (4 Performance and 6 Efficiency) cores as reported by the hardware probe, macOS 27.2.1 26B131, native arm64.
The sandbox denied the individual `sysctl` reads where [static.json](results/static.json)
shows a `system_profiler` fallback. Toolchains were rustc 1.100.0 (0ef1bc386 2026-11-09)
and Apple clang version 21.0.0 (clang-2100.3.34.2).

## Summary

| Metric | OpenKey ObjC/C++ | NeoKeyRust |
| --- | --- | --- |
| Engine median batch range (ns/key) | 42.0–69.9 | 52.0–117.8 |
| Callback median batch range (ns/key) | 92.9–478.5 | 169.2–782.7 |
| Bundle, arm64 (bytes) | 761,690 | 457,578 |
| Main executable, arm64 (bytes) | 404,328 | 454,704 |
| Bundle, universal (bytes) | 1,155,498 | 981,866 |
| Main executable, universal (bytes) | 798,136 | 978,992 |
| Engine physical / nonblank source lines | 3,337 / 2,996 | 4,535 / 4,527 |
| Platform physical / nonblank source lines | 3,323 / 2,794 | 1,991 / 1,854 |
| Explicit Rust unsafe blocks, engine / platform | n/a | 18 / 26 |

## Engine

| Case | Keys/trial | Median batch ns/key | Observed median ns/key | Observed p99 ns/key | Keys/s | Rust/C++ batch time |
| --- | --- | --- | --- | --- | --- | --- |
| telex-text-spell0-macro0 | 858 | 53.3 / 61.2 | 42 / 83 | 292 / 250 | 18,754,098 / 16,329,793 | 1.15× |
| telex-text-spell1-macro0 | 858 | 63.8 / 96.2 | 42 / 125 | 292 / 291 | 15,671,233 / 10,394,708 | 1.51× |
| telex-text-spell0-macro1 | 858 | 58.0 / 77.2 | 42 / 83 | 333 / 292 | 17,231,684 / 12,959,159 | 1.33× |
| telex-text-spell1-macro1 | 858 | 69.9 / 115.1 | 42 / 125 | 334 / 292 | 14,310,017 / 8,685,002 | 1.65× |
| telex-random-spell0-macro0 | 2048 | 46.9 / 58.1 | 42 / 83 | 250 / 333 | 21,314,683 / 17,210,084 | 1.24× |
| telex-random-spell1-macro0 | 2048 | 51.8 / 94.0 | 42 / 125 | 208 / 250 | 19,313,284 / 10,641,283 | 1.81× |
| telex-random-spell0-macro1 | 2048 | 56.1 / 76.9 | 42 / 84 | 292 / 416 | 17,841,120 / 13,006,643 | 1.37× |
| telex-random-spell1-macro1 | 2048 | 62.8 / 117.8 | 42 / 125 | 250 / 292 | 15,932,536 / 8,486,200 | 1.88× |
| vni-text-spell0-macro0 | 858 | 49.0 / 54.2 | 42 / 42 | 250 / 250 | 20,387,796 / 18,451,613 | 1.10× |
| vni-text-spell1-macro0 | 858 | 59.9 / 89.6 | 42 / 125 | 333 / 292 | 16,687,412 / 11,160,976 | 1.50× |
| vni-text-spell0-macro1 | 858 | 55.4 / 71.3 | 42 / 83 | 292 / 291 | 18,063,158 / 14,027,172 | 1.29× |
| vni-text-spell1-macro1 | 858 | 66.1 / 108.4 | 42 / 125 | 333 / 292 | 15,130,140 / 9,221,642 | 1.64× |
| vni-random-spell0-macro0 | 2048 | 42.0 / 52.0 | 42 / 42 | 250 / 292 | 23,802,606 / 19,222,647 | 1.24× |
| vni-random-spell1-macro0 | 2048 | 49.1 / 93.2 | 42 / 125 | 167 / 250 | 20,378,109 / 10,724,815 | 1.90× |
| vni-random-spell0-macro1 | 2048 | 52.3 / 72.7 | 42 / 83 | 209 / 292 | 19,102,874 / 13,764,181 | 1.39× |
| vni-random-spell1-macro1 | 2048 | 59.4 / 117.0 | 42 / 125 | 208 / 292 | 16,832,969 / 8,543,728 | 1.97× |

Each pair is **C++ / Rust**. Original C++ sources are compiled by the differential
crate's cc shim with clang `-O3 -flto=full` in Release, and Rust uses workspace Release
(`opt-level=3`, fat LTO, one codegen unit). Rust can inline Rust calls across crate
boundaries. Clang LTO covers the native objects and cannot inline into the Rust caller.
The C++ shim runs one C ABI entry per event. Both calls make the full hook observable
to the optimizer and consume its status byte without copying the hook. Engine restore-and-reset results start a new session on both sides. The
benchmark excludes engine setup, full hook copies, decoding and screen validation.

The shared differential fixture provides Vietnamese text converted to Telex and VNI,
and 2,048 seeded random events including edits, modifier flags, key-up and mouse.
Both input modes use Unicode output, with all four spell-check × macro settings.
Wrong-spelling restore is enabled. Macro entries are loaded in both engines. A separate,
untimed replay compared complete hook output and decoded screen after every event.
**23,248 engine events matched and produced 12 macro expansions**
across the per-case verification passes.

## Synthetic key-down callback

| Case | Keys/trial | Median batch ns/key | Observed median ns/key | Observed p99 ns/key | Keys/s | Rust/C++ batch time |
| --- | --- | --- | --- | --- | --- | --- |
| telex-text-batch | 858 | 366.4 / 667.4 | 84 / 167 | 1,375 / 3,542 | 2,729,225 / 1,498,256 | 1.82× |
| telex-text-step | 858 | 478.5 / 778.0 | 84 / 167 | 3,166 / 5,292 | 2,089,711 / 1,285,312 | 1.63× |
| telex-random-batch | 2026 | 126.4 / 228.9 | 83 / 125 | 1,250 / 2,958 | 7,908,934 / 4,369,525 | 1.81× |
| telex-random-step | 2026 | 170.9 / 265.6 | 83 / 125 | 3,458 / 4,208 | 5,849,861 / 3,765,505 | 1.55× |
| vni-text-batch | 858 | 363.4 / 670.3 | 83 / 166 | 1,417 / 3,584 | 2,751,835 / 1,491,959 | 1.84× |
| vni-text-step | 858 | 471.8 / 782.7 | 84 / 166 | 3,458 / 5,292 | 2,119,607 / 1,277,578 | 1.66× |
| vni-random-batch | 2026 | 92.9 / 169.2 | 83 / 125 | 1,166 / 1,833 | 10,767,031 / 5,911,014 | 1.82× |
| vni-random-step | 2026 | 107.7 / 182.9 | 83 / 125 | 1,167 / 1,792 | 9,281,146 / 5,468,286 | 1.70× |

Callback measurements use spell check and macros enabled, Unicode output, and key-down
events from both engine corpora. Each pair is **C++ / Rust**. Events are preallocated with
a fixed target PID, and the comparison covers the steady-state ordinary-app path in
both batched and per-character output modes. Layout compatibility, browser/launcher
workarounds and hotkeys are disabled. Objective-C++ runs the original
`OpenKeyCallback` with a primed PID cache and cached Backspace pair. Rust reads the
synthetic marker, PID, keycode, modifiers and repeat field, then runs the original
`Typing::process_key` and original output builder in an autorelease pool. Its timed
boundary omits app-state borrowing and the PID cache lookup. The current Rust output
builder creates fresh Backspace pairs and tags outgoing events.

Both paths resolve `CGEventTapPostEvent` to the same in-process event counter. No tap,
WindowServer event delivery, foreground IPC, application response or AX work is timed.
In this run, the sandbox denied private event-source creation and the non-null-source keyboard constructor stalled. Both sides therefore used the real Quartz keyboard constructor with a null source. A source reconstructed from a Quartz event supplied a distinct source ID for the OpenKey guard. Field reads/writes, Unicode payloads, event allocation and CF release were exercised. Private-source-specific cost is unknown.

The untimed validation records posted down/up counts and compares reconstructed UTF-16
screen snapshots after every input, including the event returned by the callback.
A mismatch at one input can affect later snapshots. The model handles text, Backspace
and printable keys and cannot model application-specific effects. The first divergent
input and screen suffix for each affected case are in the [benchmark log](results/benchmark.log).

| Case | Differing snapshots | C++ posted | Rust posted | Final text equal |
| --- | --- | --- | --- | --- |
| telex-text-batch | 0 | 1124 | 1118 | true |
| telex-text-step | 0 | 1370 | 1364 | true |
| telex-random-batch | 1772 | 424 | 424 | false |
| telex-random-step | 0 | 656 | 612 | true |
| vni-text-batch | 0 | 1124 | 1118 | true |
| vni-text-step | 0 | 1370 | 1364 | true |
| vni-random-batch | 1987 | 150 | 150 | false |
| vni-random-step | 0 | 224 | 214 | true |

At the first mismatch in 2 cases, OpenKey's batched restore emitted U+0000 for the Caps Lock–modified space. NeoKeyRust passed the physical space to the target. Both replay results are from the synthetic screen model.

## Timing method and raw samples

Each case uses 12 warmup batches, 101 timed complete batches and 21 separately
instrumented passes with an `Instant` pair per event. Side order alternates in a single
process. Each trial resets the engine and reloads the fixed macros before timing.
Median batch time is the median elapsed batch time divided by its event count.
Throughput is the reciprocal of that number. Observed p99 is the nearest-rank p99 of
individual measurements pooled per case and side. The empty clock-pair baseline was
**0 ns median / 42 ns p99** and is included in
the individual measurements. Clock granularity can produce zero-duration readings
for the empty baseline and makes small p99 differences hard to interpret. Batch timings
amortize timer overhead. Neither timing subtracts estimated overhead. The process is
unpinned and shares the machine with other processes and heterogeneous CPU cores. Repeated deterministic corpora give
warm-cache throughput and a workload-specific latency distribution.

[Raw timing CSV (gzip)](results/timings.csv.gz) · [Verification CSV](results/checks.csv) ·
[Clock baseline CSV](results/timer.csv) · [Statistics JSON](results/summary.json)

To rerun the headless suite, use `./rust/bench/run_all.sh` from the repository root.
The Rust universal size build uses the `x86_64-apple-darwin` rustup target. Outside
the sandbox, `NEOKEY_BENCH_PRIVATE_SOURCE=1 ./rust/bench/run_all.sh` supplies a private
source to the native Quartz constructor. It still intercepts posting in process.

## Before / after optimization

The optimized engine beat C++ in all 16 cases at **0.65–0.86×** the C++ median
batch time. The Rust callback beat OpenKey in all eight synthetic cases at
**0.80–0.89×**. All 24 cases were faster in each of three complete reruns on the
same Mac. The tables use median batch ns/key. Rust before comes from the
historical tables above. The two rerun columns are the medians of the three
per-run medians for each implementation. Ratios use the unrounded rerun values.

### Engine

| Case | Rust before | Rust after | C++ rerun | After Rust/C++ |
| --- | ---: | ---: | ---: | ---: |
| telex-text-spell0-macro0 | 61.2 | 43.1 | 51.4 | 0.84× |
| telex-text-spell1-macro0 | 96.2 | 47.3 | 62.2 | 0.76× |
| telex-text-spell0-macro1 | 77.2 | 48.7 | 56.8 | 0.86× |
| telex-text-spell1-macro1 | 115.1 | 53.3 | 67.7 | 0.79× |
| telex-random-spell0-macro0 | 58.1 | 39.4 | 45.9 | 0.86× |
| telex-random-spell1-macro0 | 94.0 | 34.5 | 49.5 | 0.70× |
| telex-random-spell0-macro1 | 76.9 | 45.6 | 54.9 | 0.83× |
| telex-random-spell1-macro1 | 117.8 | 40.4 | 58.8 | 0.69× |
| vni-text-spell0-macro0 | 54.2 | 39.0 | 49.0 | 0.80× |
| vni-text-spell1-macro0 | 89.6 | 43.5 | 59.8 | 0.73× |
| vni-text-spell0-macro1 | 71.3 | 44.5 | 54.6 | 0.82× |
| vni-text-spell1-macro1 | 108.4 | 49.4 | 65.9 | 0.75× |
| vni-random-spell0-macro0 | 52.0 | 34.7 | 42.4 | 0.82× |
| vni-random-spell1-macro0 | 93.2 | 31.5 | 48.4 | 0.65× |
| vni-random-spell0-macro1 | 72.7 | 40.7 | 51.2 | 0.79× |
| vni-random-spell1-macro1 | 117.0 | 37.2 | 57.5 | 0.65× |

### Synthetic key-down callback

| Case | Rust before | Rust after | C++ rerun | After Rust/C++ |
| --- | ---: | ---: | ---: | ---: |
| telex-text-batch | 667.4 | 295.9 | 348.9 | 0.85× |
| telex-text-step | 778.0 | 406.7 | 458.4 | 0.89× |
| telex-random-batch | 228.9 | 98.7 | 120.1 | 0.82× |
| telex-random-step | 265.6 | 137.5 | 166.1 | 0.83× |
| vni-text-batch | 670.3 | 296.1 | 351.9 | 0.84× |
| vni-text-step | 782.7 | 399.3 | 457.3 | 0.87× |
| vni-random-batch | 169.2 | 73.3 | 91.2 | 0.80× |
| vni-random-step | 182.9 | 85.0 | 105.5 | 0.81× |

### Where the time went

- In the fresh, unmodified baseline, enabling spell check added 35.3–40.1 ns/key
  in Rust and 4.4–11.2 ns/key in C++. Enabling macros added 16.2–21.4 ns/key in
  Rust and 5.4–10.4 ns/key in C++. These ranges are paired by corpus, input mode
  and the other setting.
- The sandbox prevented `sample` from attaching to the release profiling build.
  A temporary instrumented build replayed each selected corpus 250 times. With
  spelling on, each spelling call took 61–65 ns on Telex text and random input
  including roughly 15 ns for the clock pair. On Telex text with macros on,
  hook synchronization ran on every input at 22 ns/call. Macro lookup ran on
  194 of 858 inputs at 21 ns/call. The hook reused a reference at its public
  boundary but built new macro vectors during synchronization. Macro lookup
  already worked in place on the key.
- The follow-up probe observed about one word-break scan per Telex input, at
  23–25 ns/call including the 15–16 ns empty-clock measurement. For Telex text,
  the callback probe measured about 200 ns/key in `Typing::process_key` and
  520 ns/key in synthetic event output. OpenKey already reused its Backspace
  pair. Rust created, tagged and released two Quartz events for each Backspace.
  Instrumented times include
  probe overhead and are used to locate cost.

| Optimization | Cases measured | Saved ns/key |
| --- | --- | ---: |
| Reuse the hook’s macro buffers and read the raw character array by reference. | Engine, macro on | 11.0–15.3 |
| Index the ordered consonant rows by first key and compare the active spelling slices. | Engine, spell check on | 30.6–43.9 |
| Reuse a tagged Quartz Backspace down/up pair per output source. | Callback, all cases | 18.8–309.0 |
| Classify printable, word-break and macro-break keys with one compile-time table lookup. | Engine, all cases | 14.4–18.2 |
| Apply the same key classification change in the engine used by the callback. | Callback, all cases | 14.0–18.5 |

The savings compare consecutive release builds, one change at a time, using the
same engine and callback harness. The uninstrumented runs determine the speedups.

### Verification and rerun

After each optimization, `cargo test --workspace` passed without warnings and
`cargo run --release -p engine-diff` reported **0 mismatches over 2,645,560
keystrokes**. The benchmark's complete engine-hook checks still matched over
23,248 events, including 12 macro expansions. A separate before/after Rust
callback replay produced identical pass-through decisions, posted-event counts
and reconstructed UTF-16 snapshots at all 11,536 inputs. The two existing
Caps Lock–space divergences against OpenKey still occur.

Run the engine and synthetic callback benchmarks from `rust/` with:

```sh
export PATH=/opt/homebrew/opt/rustup/bin:$PATH
cargo run --release -p neokey-bench -- "$(mktemp -d)"
```

The callback uses null-source Quartz event construction and intercepted
posting, as in the historical run. Live private-source delivery and target-app
behavior remain unverified.

## Bundle and source scope

Bundle size sums logical bytes of regular files inside the bundle, excluding symlinks.
The arm64 OpenKey baseline was rebuilt with `ARCHS=arm64 ONLY_ACTIVE_ARCH=YES`,
Release, signing disabled and deployment target macOS 12. The supplied universal
OpenKey bundle was used when available. NeoKeyRust arm64 is the Task 2 bundle.
The measured universal Rust bundle joins that arm64 executable with a Cargo Release
x86_64 build using `lipo`, then ad-hoc signs the bundle. Both runnable apps have
different feature sets. OpenKey includes storyboards, icons and UI for preferences,
macros and conversion, plus update and login-item support.

LOC reports physical and nonblank lines, including comments and data tables, so the
numbers are reproducible without a third-party line classifier. C++ includes the
shared engine with macOS keycodes. The platform total includes ModernKey and the
OpenKey login helper source. Rust includes the engine C ABI header and both crates'
`src` trees, including inline tests. The counts exclude separate test files, the
benchmarks, the differential shim, other platforms, resources and dependencies.
Rust unsafe counts include explicit `unsafe { … }` expressions and exclude
`unsafe fn` declarations, unsafe impls and foreign dependency code.
[Per-file counts and binary metadata](results/static.json).

### Direct main-executable dependencies

- **cpp-arm64 arm64** (12) `libc++.1.dylib`, `Carbon`, `AppKit`, `ServiceManagement`, `CoreGraphics`, `Cocoa`, `EventKit`, `Foundation`, `libobjc.A.dylib`, `libSystem.B.dylib`, `ApplicationServices`, `CoreFoundation`. [otool -L](results/otool-cpp-arm64-arm64.txt)
- **rust-arm64 arm64** (17) `Carbon`, `AppKit`, `CoreVideo`, `CoreData`, `CoreText`, `CoreImage`, `CloudKit`, `QuartzCore`, `ApplicationServices`, `CoreServices`, `CoreGraphics`, `Metal`, `Foundation`, `CoreFoundation`, `libSystem.B.dylib`, `libiconv.2.dylib`, `libobjc.A.dylib`. [otool -L](results/otool-rust-arm64-arm64.txt)
- **cpp-universal x86_64** (12) `libc++.1.dylib`, `Carbon`, `AppKit`, `ServiceManagement`, `CoreGraphics`, `Cocoa`, `EventKit`, `Foundation`, `libobjc.A.dylib`, `libSystem.B.dylib`, `ApplicationServices`, `CoreFoundation`. [otool -L](results/otool-cpp-universal-x86_64.txt)
- **cpp-universal arm64** (12) `libc++.1.dylib`, `Carbon`, `AppKit`, `ServiceManagement`, `CoreGraphics`, `Cocoa`, `EventKit`, `Foundation`, `libobjc.A.dylib`, `libSystem.B.dylib`, `ApplicationServices`, `CoreFoundation`. [otool -L](results/otool-cpp-universal-arm64.txt)
- **rust-universal x86_64** (17) `Carbon`, `AppKit`, `CoreVideo`, `CoreData`, `CoreText`, `CoreImage`, `CloudKit`, `QuartzCore`, `ApplicationServices`, `CoreServices`, `CoreGraphics`, `Metal`, `Foundation`, `CoreFoundation`, `libSystem.B.dylib`, `libiconv.2.dylib`, `libobjc.A.dylib`. [otool -L](results/otool-rust-universal-x86_64.txt)
- **rust-universal arm64** (17) `Carbon`, `AppKit`, `CoreVideo`, `CoreData`, `CoreText`, `CoreImage`, `CloudKit`, `QuartzCore`, `ApplicationServices`, `CoreServices`, `CoreGraphics`, `Metal`, `Foundation`, `CoreFoundation`, `libSystem.B.dylib`, `libiconv.2.dylib`, `libobjc.A.dylib`. [otool -L](results/otool-rust-universal-arm64.txt)

## App startup and idle

Measured at 2026-11-27T22:29:06+07:00. Data and individual launch logs are in
[app_metrics.json](results/app_metrics.json) and [app_logs](results/app_logs/).
Both apps run natively on Apple Silicon. OpenKey uses the universal bundle's arm64 slice.

| Metric | OpenKey | NeoKeyRust |
| --- | --- | --- |
| Launches reaching readiness | 10/10 | 10/10 |
| Launch median / p99 (ms) | 226.14 / 931.23 | 269.33 / 286.68 |
| Ready with AppKit finished flag / large visible window | 10 / 0 | 10 / 0 |
| Enabled keyboard tap counts at readiness | 0, 1 | 0, 1 |
| RSS at 5 s (KiB) | 85,904 | 66,352 |
| Physical footprint at 5 s (KiB) | 24,145.0 | 14,304.9 |
| Threads at 5 s | 12 | 9 |
| Enabled keyboard taps at 5 s | 0 | 0 |
| RSS at 60 s (KiB) | 76,048 | 72,544 |
| Physical footprint at 60 s (KiB) | 23,409.0 | 14,544.9 |
| Threads at 60 s | 3 | 3 |
| Enabled keyboard taps at 60 s | 0 | 1 |
| Mean CPU over first 60 s of idle (% of one core) | 0.0011 | 0.0033 |

Startup is elapsed monotonic time from just before spawning the bundle executable to
confirmed readiness. The probe checks registration and either the AppKit finished-launching
flag or an on-screen app-owned window at least 100 × 40 points. It then requires 100 ms
with ≤3 ms accumulated app CPU. Polling is roughly every 10 ms. This definition includes
a permission window and is a proxy for an idle launch. Each trial starts a new process.
Filesystem caches and preferences remain in place, so later trials can benefit from warm
files. The launch p99 is the maximum with ten observations.

The first launch per app stays alive for the idle samples. RSS comes from `ps`. Physical
footprint comes from the kernel's `ri_phys_footprint`, with raw `vmmap --summary` output
and exit status retained. Thread count comes from `PROC_PIDTASKINFO`. The CPU percentage
uses the change in process user + system CPU time over the actual 60 s wall interval.

- Tap state differed, was unknown, or varied at startup. Treat these measurements as separate observations. The counts at 5 s and 60 s show whether the state changed.

## Interpretation and recommendations

- **Engine**. Rust had higher median batch time in 16/16 cases. The Rust/C++ range was 1.10× (vni-text-spell0-macro0) to 1.97× (vni-random-spell1-macro1). Rust had lower observed per-key p99 in 7/16 cases.
- **Callback**. Rust had higher median batch time in 8/8 cases. The Rust/C++ range was 1.55× (telex-random-step) to 1.84× (vni-text-batch). Rust had lower observed per-key p99 in 0/8 cases.
- The highest observed callback p99 across all cases was
  **5.29 μs**. At the requested 10–20 keys/s, either
  headless path has ample CPU capacity. These samples are orders of magnitude below
  a few-hundred-millisecond tap timeout. Live posting, target-app behavior,
  scheduling stalls and launcher AX calls can dominate perceived latency.
- Use the arm64 executable row to compare code payloads and the bundle row to
  compare what ships. Smaller bundles reflect resources and feature scope as well
  as the implementation. Compare memory and startup only with the permission
  states shown above.
- Prioritize a native, tap-enabled latency and idle comparison for both apps.
  Reproduce the Caps Lock–space restore case in a target app before changing replay behavior.
  If callback profiling identifies CGEvent creation as material, evaluate reusing
  Rust Backspace events and verify output and tap behavior before changing the path.
