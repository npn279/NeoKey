"""Paths shared by the static and AppKit measurements."""
import plistlib
from pathlib import Path

BENCH = Path(__file__).resolve().parent
ROOT = BENCH.parent.parent
TARGET = ROOT / "rust/target"
RESULTS = BENCH / "results"
SUPPLIED_OPENKEY = Path(
    "/private/tmp/claude-501/-Users-pien-Workspaces-NeoKey/"
    "bba98425-0c1d-47f6-9770-160ff60f748a/scratchpad/dd/"
    "Build/Products/Release/OpenKey.app"
)
BUILT_OPENKEY = TARGET / "bench-xcode-universal/Build/Products/Release/OpenKey.app"
CPP_ARM = TARGET / "bench-xcode-arm64/Build/Products/Release/OpenKey.app"
RUST_ARM = TARGET / "NeoKey.app"
RUST_UNIVERSAL = TARGET / "bench-universal/NeoKey.app"


def cpp_universal():
    return SUPPLIED_OPENKEY if SUPPLIED_OPENKEY.is_dir() else BUILT_OPENKEY


def app_info(bundle):
    with (bundle / "Contents/Info.plist").open("rb") as stream:
        info = plistlib.load(stream)
    executable = bundle / "Contents/MacOS" / info["CFBundleExecutable"]
    if not executable.is_file():
        raise FileNotFoundError(executable)
    return executable, info["CFBundleIdentifier"]
