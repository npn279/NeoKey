#!/bin/sh
set -eu
export PYTHONDONTWRITEBYTECODE=1
bench=$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)
exec /usr/bin/python3 "$bench/app_metrics.py" "$@"
