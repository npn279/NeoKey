#!/bin/sh
set -eu

# Run in a macOS desktop session with Spotlight closed, one NeoKey app enabled
# in Vietnamese Telex mode, and Accessibility permission for the terminal.
script_dir=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
build_dir=$(mktemp -d "${TMPDIR:-/tmp}/neokey-spotlight.XXXXXX")
trap 'rm -rf "$build_dir"' EXIT

xcrun clang -fobjc-arc -Wall -Wextra -framework AppKit -framework ApplicationServices \
    "$script_dir/spotlight_oo.m" -o "$build_dir/spotlight_oo"
"$build_dir/spotlight_oo"
