"""Measure the shipped file layouts, direct Mach-O dependencies, and source trees."""
import datetime as dt
import hashlib
import json
import os
import re
import subprocess

from paths import CPP_ARM, RESULTS, ROOT, RUST_ARM, RUST_UNIVERSAL, app_info, cpp_universal


def command(*argv, required=True):
    result = subprocess.run(argv, capture_output=True, text=True, timeout=30)
    if required:
        result.check_returncode()
    return result.stdout.strip() if result.returncode == 0 else None


def machine():
    values = {
        name: command("/usr/sbin/sysctl", "-n", name, required=False)
        for name in ("hw.model", "machdep.cpu.brand_string", "hw.ncpu")
    }
    profile = command("/usr/sbin/system_profiler", "SPHardwareDataType", required=False) or ""
    for key, label in (
        ("hw.model", "Model Identifier"),
        ("machdep.cpu.brand_string", "Chip"),
        ("hw.ncpu", "Total Number of Cores"),
    ):
        if values[key] is not None:
            continue
        match = re.search(rf"^\s*{label}: (.*)$", profile, re.M)
        if match:
            values[key] = match[1]
            values[key + "_source"] = "system_profiler fallback (sysctl unavailable)"
    values.update(
        arch=os.uname().machine,
        macos=command("/usr/bin/sw_vers"),
        rustc=command("rustc", "-vV"),
        clang=command("/usr/bin/xcrun", "clang", "--version"),
        xcode=command("/usr/bin/xcodebuild", "-version"),
    )
    return values


def binary_metrics(bundle, expected_arches, name):
    if not bundle.is_dir():
        return {"missing": str(bundle)}
    executable, identifier = app_info(bundle)
    arches = command("/usr/bin/lipo", "-archs", str(executable)).split()
    if set(arches) != set(expected_arches):
        raise ValueError(f"{name}: expected {expected_arches}, got {arches}")
    dylibs = {}
    for arch in arches:
        report = command("/usr/bin/otool", "-arch", arch, "-L", str(executable))
        (RESULTS / f"otool-{name}-{arch}.txt").write_text(report + "\n")
        dylibs[arch] = [line.strip().split(" (", 1)[0] for line in report.splitlines()[1:]]
    return {
        "bundle": str(bundle),
        "bundle_identifier": identifier,
        "bundle_bytes": sum(
            path.stat().st_size
            for path in bundle.rglob("*")
            if not path.is_symlink() and path.is_file()
        ),
        "exe_bytes": executable.stat().st_size,
        "exe_sha256": hashlib.sha256(executable.read_bytes()).hexdigest(),
        "archs": arches,
        "dylibs": dylibs,
    }


# Skip Rust comments and literals before counting explicit unsafe expressions.
# Block comments can nest. A normal character literal consumes one character or
# escape, so lifetime apostrophes remain ordinary source.
RUST_TOKEN = re.compile(
    r'/\*|//[^\n]*|(?:b|c)?r(?P<hashes>\#*)"(?s:.*?)"(?P=hashes)'
    r'|"(?:\\.|[^"\\])*"|\'(?:\\(?:u\{[\da-fA-F_]+\}|x[\da-fA-F]{2}|.)|[^\'\\\n])\''
    r'|(?P<unsafe>\bunsafe\b)|(?P<brace>\{)|(?P<other>\S)'
)
COMMENT_EDGE = re.compile(r'/\*|\*/')


def unsafe_blocks(source):
    count, pos = 0, 0
    after_unsafe = False
    while match := RUST_TOKEN.search(source, pos):
        token = match.group()
        pos = match.end()
        if token == "/*":
            depth = 1
            while depth:
                edge = COMMENT_EDGE.search(source, pos)
                if edge is None:
                    raise ValueError("unclosed Rust block comment")
                depth += 1 if edge.group() == "/*" else -1
                pos = edge.end()
        elif not token.startswith("//"):
            count += bool(after_unsafe and match["brace"])
            after_unsafe = bool(match["unsafe"])
    return count


def source_metrics(paths, suffixes):
    files = {}
    for folder in paths:
        for path in sorted((ROOT / folder).rglob("*")):
            if path.suffix not in suffixes:
                continue
            if path.name in {"linux.h", "win32.h"} and path.parent.name == "platforms":
                continue
            source = path.read_text(encoding="utf-8-sig")
            lines = source.splitlines()
            row = {"physical_lines": len(lines), "nonblank_lines": sum(bool(line.strip()) for line in lines)}
            if path.suffix == ".rs":
                row["unsafe_blocks"] = unsafe_blocks(source)
            files[str(path.relative_to(ROOT))] = row
    return {
        "files": files,
        **{field: sum(row.get(field, 0) for row in files.values())
           for field in ("physical_lines", "nonblank_lines", "unsafe_blocks")},
    }


def main():
    RESULTS.mkdir(exist_ok=True)
    app_specs = [
        ("cpp-arm64", CPP_ARM, ["arm64"]),
        ("rust-arm64", RUST_ARM, ["arm64"]),
        ("cpp-universal", cpp_universal(), ["arm64", "x86_64"]),
        ("rust-universal", RUST_UNIVERSAL, ["arm64", "x86_64"]),
    ]
    c_family = {".c", ".cc", ".cpp", ".h", ".m", ".mm"}
    out = {
        "measured_at": dt.datetime.now().astimezone().isoformat(timespec="seconds"),
        "machine": machine(),
        "binaries": {name: binary_metrics(bundle, arches, name) for name, bundle, arches in app_specs},
        "source": {
            "cpp-engine": source_metrics(["Sources/OpenKey/engine"], c_family),
            "cpp-platform": source_metrics(
                ["Sources/OpenKey/macOS/ModernKey", "Sources/OpenKey/macOS/OpenKeyHelper"], c_family
            ),
            "rust-engine": source_metrics(["rust/neokey-engine/src", "rust/neokey-engine/include"], {".rs", ".h"}),
            "rust-platform": source_metrics(["rust/neokey-mac/src"], {".rs"}),
        },
    }
    (RESULTS / "static.json").write_text(json.dumps(out, indent=2) + "\n")


if __name__ == "__main__":
    main()
