#!/bin/sh
set -eu
# Tested under d-i before destructive partman confirmations. A second disk,
# wrong target or missing VM serial causes failure, never first-disk selection.
sys=${FUNGOS_TEST_SYS_BLOCK:-/sys/block}
count=0
for disk in "$sys"/*; do
  [ -e "$disk" ] || continue
  case "${disk##*/}" in loop*|ram*|sr*|fd*) continue;; esac
  count=$((count + 1))
  [ "${disk##*/}" = vda ] || { echo 'unexpected writable disk' >&2; exit 1; }
  serial="$disk/serial"
  [ -r "$serial" ] || serial="$disk/device/serial"
  [ "$(cat "$serial")" = FUNGOS_DISPOSABLE ] || { echo 'wrong disposable disk identity' >&2; exit 1; }
  [ "$(cat "$disk/size")" = 16777216 ] || { echo 'wrong disposable disk size' >&2; exit 1; }
done
[ "$count" = 1 ] || { echo 'ambiguous or absent disposable disk' >&2; exit 1; }
