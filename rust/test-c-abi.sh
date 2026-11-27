#!/bin/sh
set -eu
cd "$(dirname "$0")"
export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
cargo build --release -p neokey-engine
scratch=$(mktemp -d "${TMPDIR:-/tmp}/neokey-c-abi.XXXXXX")
trap 'rm -rf "$scratch"' EXIT
case "$(uname -s)" in
  Darwin) libs="-framework Security -framework CoreFoundation -liconv -lSystem" ;;
  *) libs="-ldl -lpthread -lm" ;;
esac
# shellcheck disable=SC2086
cc -std=c11 -Wall -Wextra -Werror -I neokey-engine/include neokey-engine/tests/c_abi.c target/release/libneokey_engine.a $libs -o "$scratch/c-abi"
"$scratch/c-abi"
echo "C ABI static-library smoke test passed"
