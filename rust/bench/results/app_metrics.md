## App startup and idle

Measured at 2026-11-27T22:29:06+07:00. Data and individual launch logs are in
[app_metrics.json](app_metrics.json) and [app_logs](app_logs/).
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