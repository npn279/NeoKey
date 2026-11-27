"""Render RESULTS.md from the machine-readable measurements."""
import csv
import gzip
import json
import math
import statistics
from collections import defaultdict

from paths import BENCH, RESULTS


def p99(values):
    return sorted(values)[math.ceil(len(values) * 0.99) - 1]


def table(headers, rows):
    return "\n".join(
        "| " + " | ".join(map(str, row)) + " |"
        for row in (headers, ["---"] * len(headers), *rows)
    )


def timing_summary():
    samples = defaultdict(lambda: defaultdict(list))
    with gzip.open(RESULTS / "timings.csv.gz", "rt") as stream:
        for row in csv.DictReader(stream):
            key = row["scope"], row["case"], row["side"]
            samples[key][row["sample"]].append(int(row["ns"]) / int(row["keys"]))
    return {
        group: {
            "batch_median_ns_per_key": (median := statistics.median(v["batch"])),
            "keys_per_second": 1e9 / median,
            "observed_key_median_ns": statistics.median(v["key"]),
            "observed_key_p99_ns": p99(v["key"]),
            "batch_samples": len(v["batch"]),
            "key_samples": len(v["key"]),
        }
        for group, v in samples.items()
    }


def app_fragment(payload, binaries=None, prefix="results/"):
    if payload is None:
        return """## App startup and idle

Pending. AppKit launches are blocked by the Codex sandbox. Run `./rust/bench/app_metrics.sh`
from the logged-in macOS desktop. It produces [app_metrics.json](results/app_metrics.json)
and [app_metrics.md](results/app_metrics.md) and refreshes this report.

The script records enabled keyboard taps. OpenKey enters its permission dialog before
creating its status item when untrusted. NeoKeyRust creates its status item before prompting
and retries the tap. Even with both taps disabled, those startup modes can yield different
idle behavior. No runtime measurements are available for either app in this report."""

    apps = payload["apps"]
    rows = defaultdict(list)
    notes = []
    modes = []
    for side, static_key in (("cpp", "cpp-universal"), ("rust", "rust-arm64")):
        app = apps[side]
        attempts = app["launches"]
        starts = [run["startup_ms"] for run in attempts if "startup_ms" in run]
        enabled = sorted({run["ready"]["enabled_keyboard_taps"] for run in attempts if "ready" in run},
                         key=lambda value: -1 if value is None else value)
        modes.append(enabled)
        idle = next((run["idle"] for run in attempts if "idle" in run), None)
        rows["Launches reaching readiness"].append(f"{len(starts)}/{len(attempts)}")
        rows["Launch median / p99 (ms)"].append(
            f"{statistics.median(starts):.2f} / {p99(starts):.2f}" if starts else "unavailable"
        )
        finished = sum(run["ready"]["finished_launching"] for run in attempts if "ready" in run)
        windows = sum(run["ready"]["large_window_visible"] for run in attempts if "ready" in run)
        rows["Ready with AppKit finished flag / large visible window"].append(f"{finished} / {windows}")
        rows["Enabled keyboard tap counts at readiness"].append(
            ", ".join("unknown" if v is None else str(v) for v in enabled) or "unavailable"
        )
        for index, seconds in enumerate((5, 60)):
            sample = idle["samples"][index] if idle else {}
            rss = sample.get("rss_kib")
            phys = sample.get("physical_footprint_bytes")
            taps = sample.get("enabled_keyboard_taps")
            rows[f"RSS at {seconds} s (KiB)"].append("unavailable" if rss is None else f"{rss:,}")
            rows[f"Physical footprint at {seconds} s (KiB)"].append(
                "unavailable" if phys is None else f"{phys / 1024:,.1f}"
            )
            rows[f"Threads at {seconds} s"].append(sample.get("threads", "unavailable"))
            rows[f"Enabled keyboard taps at {seconds} s"].append("unknown" if taps is None else taps)
        rows["Mean CPU over first 60 s of idle (% of one core)"].append(
            f"{idle['cpu_pct_one_core']:.4f}" if idle else "unavailable"
        )
        if binaries and app["executable_sha256"] != binaries[static_key].get("exe_sha256"):
            notes.append(f"{side} runtime executable SHA-256 differs from the binary in the size table.")
        for run in attempts:
            if "error" in run:
                notes.append(f"{side} launch {run['trial']} — {run['error']}")
    if modes == [[1], [1]]:
        notes.append("Each successful launch had one enabled keyboard tap at readiness. Typing was not exercised.")
    elif modes == [[0], [0]]:
        notes.append("Both apps had no enabled keyboard tap at readiness. OpenKey can idle in its permission "
                     "dialog before creating its status item. NeoKeyRust creates its status item before prompting.")
    else:
        notes.append("Tap state differed, was unknown, or varied at startup. Treat these measurements as "
                     "separate observations. The counts at 5 s and 60 s show whether the state changed.")
    return f"""## App startup and idle

Measured at {payload['measured_at']}. Data and individual launch logs are in
[app_metrics.json]({prefix}app_metrics.json) and [app_logs]({prefix}app_logs/).
Both apps run natively on Apple Silicon. OpenKey uses the universal bundle's arm64 slice.

{table(("Metric", "OpenKey", "NeoKeyRust"), [(name, *pair) for name, pair in rows.items()])}

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

""" + "\n".join(f"- {note}" for note in notes)


def render_report():
    static = json.loads((RESULTS / "static.json").read_text())
    binaries, source, host = (static[key] for key in ("binaries", "source", "machine"))
    app_path = RESULTS / "app_metrics.json"
    app = json.loads(app_path.read_text()) if app_path.is_file() else None
    timing = timing_summary()
    checked = list(csv.DictReader((RESULTS / "checks.csv").read_text().splitlines()))
    callback_source = (RESULTS / "callback_source.txt").read_text().strip()
    results = [
        {"scope": scope, "case": case, "side": side, **stats}
        for (scope, case, side), stats in timing.items()
    ]
    timer = [int(row["ns"]) for row in csv.DictReader((RESULTS / "timer.csv").read_text().splitlines())]
    (RESULTS / "summary.json").write_text(json.dumps({
        "timings": results,
        "callback_source": callback_source,
        "timer_baseline_ns": {"median": statistics.median(timer), "p99": p99(timer)},
    }, indent=2) + "\n")

    def fmt_pair(cpp, rust, key, digits):
        return " / ".join(f"{v[key]:,.{digits}f}" for v in (cpp, rust))

    timing_rows = defaultdict(list)
    ratios = defaultdict(list)
    all_p99 = defaultdict(list)
    batch_times = defaultdict(list)
    rust_lower_p99 = defaultdict(int)
    for row in checked:
        scope, case = row["scope"], row["case"]
        cpp, rust = (timing[scope, case, side] for side in ("cpp", "rust"))
        ratio = rust["batch_median_ns_per_key"] / cpp["batch_median_ns_per_key"]
        ratios[scope].append((ratio, case))
        all_p99[scope].extend(v["observed_key_p99_ns"] for v in (cpp, rust))
        for side, stats in (("cpp", cpp), ("rust", rust)):
            batch_times[scope, side].append(stats["batch_median_ns_per_key"])
        rust_lower_p99[scope] += rust["observed_key_p99_ns"] < cpp["observed_key_p99_ns"]
        timing_rows[scope].append([
            case,
            row["keys"],
            fmt_pair(cpp, rust, "batch_median_ns_per_key", 1),
            fmt_pair(cpp, rust, "observed_key_median_ns", 0),
            fmt_pair(cpp, rust, "observed_key_p99_ns", 0),
            fmt_pair(cpp, rust, "keys_per_second", 0),
            f"{ratio:.2f}×",
        ])
    headers = ("Case", "Keys/trial", "Median batch ns/key", "Observed median ns/key",
               "Observed p99 ns/key", "Keys/s", "Rust/C++ batch time")
    compare_rows = [
        (row["case"], row["mismatched_callback_snapshots"], row["cpp_posted"], row["rust_posted"],
         row["final_screen_equal"])
        for row in checked if row["scope"] == "callback"
    ]
    log_path = RESULTS / "benchmark.log"
    log_lines = log_path.read_text().splitlines() if log_path.is_file() else []
    caps_space = [line for line in log_lines if "(virtual key 49, flags 0x10000)" in line
                  and r'\0", NeoKeyRust ' in line and line.endswith(' "')]
    divergence = ""
    if caps_space:
        divergence = (
            f"At the first mismatch in {len(caps_space)} cases, OpenKey's batched restore "
            "emitted U+0000 for the Caps Lock–modified space. NeoKeyRust passed the physical "
            "space to the target. Both replay results are from the synthetic screen model.\n\n"
        )
    verification = sum(int(row["keys"]) for row in checked if row["scope"] == "engine")
    macro_hits = sum(int(row["macro_hits"]) for row in checked if row["scope"] == "engine")
    resource_rows = []
    for arch in ("arm64", "universal"):
        cpp, rust = [binaries[f"{side}-{arch}"] for side in ("cpp", "rust")]
        for metric, label in (("bundle_bytes", "Bundle"), ("exe_bytes", "Main executable")):
            resource_rows.append((f"{label}, {arch} (bytes)", *[
                f"{app[metric]:,}" if metric in app else "unavailable" for app in (cpp, rust)
            ]))
    for part in ("engine", "platform"):
        cpp, rust = [source[f"{side}-{part}"] for side in ("cpp", "rust")]
        resource_rows.append((f"{part.title()} physical / nonblank source lines", *[
            f"{side['physical_lines']:,} / {side['nonblank_lines']:,}" for side in (cpp, rust)
        ]))
    resource_rows.append(("Explicit Rust unsafe blocks, engine / platform", "n/a",
                          f"{source['rust-engine']['unsafe_blocks']} / {source['rust-platform']['unsafe_blocks']}"))

    descriptions = []
    for scope, group in ratios.items():
        lo, hi = min(group), max(group)
        slow = sum(ratio > 1 for ratio, _ in group)
        descriptions.append(
            f"- **{scope.title()}**. Rust had higher median batch time in {slow}/{len(group)} cases. "
            f"The Rust/C++ range was {lo[0]:.2f}× ({lo[1]}) to {hi[0]:.2f}× ({hi[1]}). "
            f"Rust had lower observed per-key p99 in {rust_lower_p99[scope]}/{len(group)} cases."
        )
    linked = []
    for name, data in binaries.items():
        if "dylibs" not in data:
            linked.append(f"- **{name}** unavailable")
            continue
        for arch, dylibs in data["dylibs"].items():
            names = ", ".join(f"`{path.split('/')[-1]}`" for path in dylibs)
            linked.append(f"- **{name} {arch}** ({len(dylibs)}) {names}. [otool -L](results/otool-{name}-{arch}.txt)")

    macos = " ".join(line.split(":", 1)[-1].strip() for line in host["macos"].splitlines())
    source_note = (
        "In this run, the sandbox denied private event-source creation and the non-null-source "
        "keyboard constructor stalled. Both sides therefore used the real Quartz keyboard "
        "constructor with a null source. A source reconstructed from a Quartz event supplied "
        "a distinct source ID for the OpenKey guard. Field reads/writes, Unicode payloads, "
        "event allocation and CF release were exercised. Private-source-specific cost is unknown."
        if callback_source == "null" else
        "This run supplied each implementation's private Core Graphics source to the native "
        "Quartz keyboard constructor. Inputs were prebuilt with a null source."
    )
    def range_ns(values):
        return f"{min(values):,.1f}–{max(values):,.1f}"

    summary = table(
        ("Metric", "OpenKey ObjC/C++", "NeoKeyRust"),
        [
            *[
                (f"{scope.title()} median batch range (ns/key)", *[
                    range_ns(batch_times[scope, side]) for side in ("cpp", "rust")
                ])
                for scope in ("engine", "callback")
            ],
            *resource_rows,
        ],
    )
    report = f"""# NeoKey efficiency measurements

Measured {static['measured_at']} on **{host['hw.model']}**, {host['machdep.cpu.brand_string']},
{host['hw.ncpu']} cores as reported by the hardware probe, {macos}, native {host['arch']}.
The sandbox denied the individual `sysctl` reads where [static.json](results/static.json)
shows a `system_profiler` fallback. Toolchains were {host['rustc'].splitlines()[0]}
and {host['clang'].splitlines()[0]}.

## Summary

{summary}

## Engine

{table(headers, timing_rows['engine'])}

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
**{verification:,} engine events matched and produced {macro_hits} macro expansions**
across the per-case verification passes.

## Synthetic key-down callback

{table(headers, timing_rows['callback'])}

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
{source_note}

The untimed validation records posted down/up counts and compares reconstructed UTF-16
screen snapshots after every input, including the event returned by the callback.
A mismatch at one input can affect later snapshots. The model handles text, Backspace
and printable keys and cannot model application-specific effects. The first divergent
input and screen suffix for each affected case are in the [benchmark log](results/benchmark.log).

{table(("Case", "Differing snapshots", "C++ posted", "Rust posted", "Final text equal"), compare_rows)}

{divergence}## Timing method and raw samples

Each case uses 12 warmup batches, 101 timed complete batches and 21 separately
instrumented passes with an `Instant` pair per event. Side order alternates in a single
process. Each trial resets the engine and reloads the fixed macros before timing.
Median batch time is the median elapsed batch time divided by its event count.
Throughput is the reciprocal of that number. Observed p99 is the nearest-rank p99 of
individual measurements pooled per case and side. The empty clock-pair baseline was
**{statistics.median(timer):.0f} ns median / {p99(timer)} ns p99** and is included in
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
Rust unsafe counts include explicit `unsafe {{ … }}` expressions and exclude
`unsafe fn` declarations, unsafe impls and foreign dependency code.
[Per-file counts and binary metadata](results/static.json).

### Direct main-executable dependencies

""" + "\n".join(linked) + "\n\n" + app_fragment(app, binaries) + f"""

## Interpretation and recommendations

""" + "\n".join(descriptions) + f"""
- The highest observed callback p99 across all cases was
  **{max(all_p99['callback']) / 1000:,.2f} μs**. At the requested 10–20 keys/s, either
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
"""
    (BENCH / "RESULTS.md").write_text(report)


if __name__ == "__main__":
    render_report()
