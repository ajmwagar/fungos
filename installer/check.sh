#!/bin/sh
set -eu
dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
for file in "$dir"/*.sh; do sh -n "$file"; done
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT HUP INT TERM
mkdir -p "$tmp/vda/device"
printf 'FUNGOS_DISPOSABLE\n' > "$tmp/vda/device/serial"
printf '16777216\n' > "$tmp/vda/size"
FUNGOS_TEST_SYS_BLOCK="$tmp" sh "$dir/guard-disk.sh"
mkdir "$tmp/vdb"
if FUNGOS_TEST_SYS_BLOCK="$tmp" sh "$dir/guard-disk.sh" 2>/dev/null; then echo 'ambiguous disk accepted' >&2; exit 1; fi
rmdir "$tmp/vdb"
printf 'REAL_DISK\n' > "$tmp/vda/device/serial"
if FUNGOS_TEST_SYS_BLOCK="$tmp" sh "$dir/guard-disk.sh" 2>/dev/null; then echo 'wrong identity accepted' >&2; exit 1; fi
if grep -Eq 'partman|password|authorized_keys|claim|token' "$dir/interactive.preseed"; then
  # Comments describe absence; examine only actual answers.
  if sed '/^#/d' "$dir/interactive.preseed" | grep -Eq 'partman|password|authorized_keys|claim|token'; then echo 'unsafe public preseed' >&2; exit 1; fi
fi
if [ -n "${DEBIAN_TEST_ISO:-}" ]; then
  mkdir -p "$tmp/source/fungOS/installer"
  cp -a "$dir/../base" "$tmp/source/fungOS/base"
  cp "$dir/build.sh" "$tmp/source/fungOS/installer/"
  : > "$tmp/source/fungOS/base/rootfs/.env"
  if sh "$tmp/source/fungOS/installer/build.sh" "$DEBIAN_TEST_ISO" "$tmp/unsafe.iso" > "$tmp/rejection.log" 2>&1; then
    echo 'credential overlay accepted' >&2; exit 1
  fi
  grep -q 'credential state found' "$tmp/rejection.log"
  [ ! -e "$tmp/unsafe.iso" ]
fi
echo 'installer syntax, positive disk guard, ambiguity rejection, identity rejection, public preseed safety: passed'
