#!/bin/sh
set -eu

workspace=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
target="$workspace/target"
bundle="$target/NeoKey.app"

cd "$workspace"
cargo build --locked --release -p neokey-mac --target-dir "$target"
mkdir -p "$bundle/Contents/MacOS" "$bundle/Contents/Resources"
install -m 755 "$target/release/neokey-mac" "$bundle/Contents/MacOS/neokey-mac"
install -m 644 "$workspace/neokey-mac/Info.plist" "$bundle/Contents/Info.plist"
install -m 644 "$workspace/neokey-mac/assets/AppIcon.icns" "$bundle/Contents/Resources/AppIcon.icns"
codesign -s - --force "$bundle"
codesign --verify --strict "$bundle"
printf '%s\n' "$bundle"
