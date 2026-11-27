"""Run only from the logged-in macOS desktop. Each app process is our own child."""
import datetime as dt
import hashlib
import json
import os
import select
import signal
import statistics
import subprocess
import sys
import time
from contextlib import contextmanager

from paths import BENCH, RESULTS, TARGET, RUST_ARM, app_info, cpp_universal

LAUNCHES = 10
READY_TIMEOUT = 20
QUIET_PERIOD = 0.1
QUIET_CPU_NS = 3_000_000


@contextmanager
def child(argv, **kwargs):
    process = subprocess.Popen(argv, **kwargs)
    try:
        yield process
    finally:
        if process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=3)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=3)


class Probe:
    def __init__(self, process):
        self.process = process

    def query(self, line):
        self.process.stdin.write(line + "\n")
        self.process.stdin.flush()
        if not select.select([self.process.stdout], [], [], 3)[0]:
            raise RuntimeError("AppKit probe timed out")
        reply = self.process.stdout.readline().strip()
        if not reply:
            raise RuntimeError("AppKit probe exited")
        return reply

    def snapshot(self, pid, window=False, taps=False):
        before = time.monotonic()
        reply = self.query(f"pid {pid} {int(window)} {int(taps)}")
        after = time.monotonic()
        if reply == "unavailable":
            return None
        registered, finished, visible, cpu, footprint, threads, enabled = map(int, reply.split())
        return dict(
            monotonic_s=(before + after) / 2,
            registered=bool(registered),
            finished_launching=bool(finished),
            large_window_visible=bool(visible),
            cpu_ns=cpu,
            physical_footprint_bytes=footprint,
            threads=threads,
            enabled_keyboard_taps=None if enabled < 0 else enabled,
        )


def startup(probe, process, start):
    quiet_start = None
    while time.monotonic() - start < READY_TIMEOUT:
        if process.poll() is not None:
            raise RuntimeError(f"app exited before readiness (exit {process.returncode})")
        sample = probe.snapshot(process.pid, window=True)
        if sample and sample["registered"] and (
            sample["finished_launching"] or sample["large_window_visible"]
        ):
            if quiet_start is None or sample["cpu_ns"] - quiet_start["cpu_ns"] > QUIET_CPU_NS:
                quiet_start = sample
            elif sample["monotonic_s"] - quiet_start["monotonic_s"] >= QUIET_PERIOD:
                return sample
        else:
            quiet_start = None
        time.sleep(0.01)
    raise RuntimeError(f"app did not reach the launch/idle criterion within {READY_TIMEOUT}s")


def command_to_file(argv, path):
    with path.open("w") as stream:
        try:
            return subprocess.run(argv, stdout=stream, stderr=subprocess.STDOUT, timeout=15).returncode
        except subprocess.TimeoutExpired:
            stream.write("\nCommand timed out after 15 seconds.\n")
            return None


def idle_samples(probe, process, zero, logdir):
    samples = []
    for seconds in (5, 60):
        time.sleep(max(0, zero["monotonic_s"] + seconds - time.monotonic()))
        sample = probe.snapshot(process.pid, window=True, taps=True)
        if sample is None:
            raise RuntimeError("app disappeared during the idle interval")
        sample["target_seconds"] = seconds
        sample["elapsed_seconds"] = sample["monotonic_s"] - zero["monotonic_s"]
        rss = subprocess.run(
            ["/bin/ps", "-o", "rss=", "-p", str(process.pid)],
            capture_output=True, text=True, timeout=5,
        )
        sample["rss_kib"] = int(rss.stdout.strip()) if rss.stdout.strip().isdigit() else None
        sample["vmmap_exit"] = command_to_file(
            ["/usr/bin/vmmap", "--summary", str(process.pid)], logdir / f"vmmap-{seconds}s.txt"
        )
        samples.append(sample)
    last = samples[-1]
    cpu = (last["cpu_ns"] - zero["cpu_ns"]) / 1e9 / last["elapsed_seconds"] * 100
    return {"samples": samples, "cpu_pct_one_core": cpu, "cpu_interval_s": last["elapsed_seconds"]}


def launch(probe, executable, trial, logdir):
    logdir.mkdir(parents=True, exist_ok=True)
    with (logdir / "app.log").open("w") as log:
        start = time.monotonic()
        with child([str(executable)], stdout=log, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL) as proc:
            result = {"trial": trial, "pid": proc.pid}
            try:
                ready = startup(probe, proc, start)
                first = probe.snapshot(proc.pid, window=True, taps=True)
                if not first:
                    raise RuntimeError("app exited immediately after readiness")
                result.update(startup_ms=(ready["monotonic_s"] - start) * 1000, ready=first)
                if trial == 1:
                    result["idle"] = idle_samples(probe, proc, first, logdir)
            except (RuntimeError, subprocess.SubprocessError) as error:
                result["error"] = str(error)
            return result


def main():
    RESULTS.mkdir(exist_ok=True)
    raw = RESULTS / "app_logs"
    raw.mkdir(exist_ok=True)
    TARGET.mkdir(exist_ok=True)
    probe_path = TARGET / "neokey-bench-app-probe"
    apps = {"cpp": cpp_universal(), "rust": RUST_ARM}
    info = {side: app_info(bundle) for side, bundle in apps.items()}
    subprocess.run(
        ["/usr/bin/xcrun", "clang", "-O2", "-fobjc-arc", "-framework", "AppKit",
         "-framework", "CoreGraphics", str(BENCH / "app_probe.m"), "-o", str(probe_path)],
        check=True, timeout=60,
    )
    payload = {
        "measured_at": dt.datetime.now().astimezone().isoformat(timespec="seconds"),
        "macos": subprocess.check_output(["/usr/bin/sw_vers"], text=True).strip(),
        "machine": os.uname().machine,
        "requested_launches_per_app": LAUNCHES,
        "apps": {},
    }
    with child([str(probe_path)], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True) as process:
        probe = Probe(process)
        for side, (exe, identifier) in info.items():
            if int(probe.query(f"existing {identifier}")):
                raise RuntimeError(f"Quit the existing {identifier} instance before running the benchmark.")
            payload["apps"][side] = dict(
                bundle=str(apps[side]),
                executable_sha256=hashlib.sha256(exe.read_bytes()).hexdigest(),
                launches=[],
            )

        for trial in range(1, LAUNCHES + 1):
            order = ("cpp", "rust") if trial % 2 else ("rust", "cpp")
            for side in order:
                print(f"{side} launch {trial}/{LAUNCHES}", flush=True)
                result = launch(probe, info[side][0], trial, raw / side / str(trial))
                payload["apps"][side]["launches"].append(result)
                time.sleep(0.25)
    json_path = RESULTS / "app_metrics.json"
    temp = json_path.with_suffix(".json.tmp")
    temp.write_text(json.dumps(payload, indent=2) + "\n")
    temp.replace(json_path)
    from report import app_fragment, render_report

    (RESULTS / "app_metrics.md").write_text(app_fragment(payload, prefix=""))
    if all((RESULTS / name).is_file() for name in ("timings.csv.gz", "checks.csv", "static.json", "timer.csv", "callback_source.txt")):
        render_report()
    print(f"Wrote {json_path}", flush=True)
    return int(any("error" in run for app in payload["apps"].values() for run in app["launches"]))


if __name__ == "__main__":
    # SIGTERM and Ctrl-C unwind the child context and reap only the processes we own.
    def terminate(_signal, _frame):
        raise KeyboardInterrupt

    signal.signal(signal.SIGTERM, terminate)
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        sys.exit(130)
