#!/bin/bash
set -euo pipefail
export PATH="/opt/homebrew/opt/rustup/bin:$PATH" PYTHONDONTWRITEBYTECODE=1

bench=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
rust=$(dirname "$bench")
root=$(dirname "$rust")
export CARGO_TARGET_DIR="$rust/target"
if [[ -z ${CARGO_HOME:-} && -d /private/tmp/neokey-cargo-home ]]; then
    export CARGO_HOME=/private/tmp/neokey-cargo-home
fi

results="$bench/results"
mkdir -p "$results"
stage=$(mktemp -d "$results/.run.XXXXXX")
trap 'rm -rf -- "$stage"' EXIT

cd "$root"
build_openkey() {
    local archs=$1 active=$2 output=$3
    if ! xcodebuild -project Sources/OpenKey/macOS/OpenKey.xcodeproj -scheme OpenKey \
        -configuration Release -derivedDataPath "$CARGO_TARGET_DIR/bench-xcode-$output" \
        build CODE_SIGNING_ALLOWED=NO MACOSX_DEPLOYMENT_TARGET=12.0 \
        "ARCHS=$archs" "ONLY_ACTIVE_ARCH=$active" >"$CARGO_TARGET_DIR/bench-xcode-$output.log" 2>&1; then
        tail -60 "$CARGO_TARGET_DIR/bench-xcode-$output.log" >&2
        exit 1
    fi
}

echo 'Building arm64 OpenKey Release'
build_openkey arm64 YES arm64
if ! (cd "$bench" && /usr/bin/python3 -c 'from paths import SUPPLIED_OPENKEY; raise SystemExit(not SUPPLIED_OPENKEY.is_dir())'); then
    echo 'Building universal OpenKey Release'
    build_openkey 'arm64 x86_64' NO universal
fi

cd "$rust"
cargo build --locked --release -p neokey-bench
if [[ ! -x target/NeoKeyRust.app/Contents/MacOS/neokey-mac ]]; then
    ./neokey-mac/bundle.sh
fi

if rustup target list --installed | grep -qx x86_64-apple-darwin; then
    cargo build --locked --release -p neokey-mac --target x86_64-apple-darwin
    bundle="$CARGO_TARGET_DIR/bench-universal/NeoKeyRust.app"
    mkdir -p "$bundle/Contents/MacOS"
    install -m 644 target/NeoKeyRust.app/Contents/Info.plist "$bundle/Contents/Info.plist"
    xcrun lipo -create target/NeoKeyRust.app/Contents/MacOS/neokey-mac \
        target/x86_64-apple-darwin/release/neokey-mac -output "$bundle/Contents/MacOS/neokey-mac"
    codesign --force --sign - "$bundle"
    codesign --verify --strict "$bundle"
else
    echo 'Rust x86_64 target unavailable. Universal Rust measurement requires rust-std for x86_64-apple-darwin.' >&2
fi

/usr/bin/python3 "$bench/static_metrics.py"
callback_command=("$CARGO_TARGET_DIR/release/neokey-bench" "$stage")
if [[ ${NEOKEY_BENCH_PRIVATE_SOURCE:-0} == 1 ]]; then
    callback_command+=(--private-source)
fi
/usr/bin/python3 -c 'import subprocess, sys; subprocess.run(sys.argv[1:], check=True, timeout=180)' \
    "${callback_command[@]}" 2>&1 | tee "$stage/benchmark.log"
gzip -n "$stage/timings.csv"
for name in timings.csv.gz checks.csv timer.csv callback_source.txt benchmark.log; do
    mv "$stage/$name" "$results/$name"
done
/usr/bin/python3 "$bench/report.py"
