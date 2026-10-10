#!/bin/sh
# Only public, repository-owned system overlay nodes are normalized. Never
# recursively change ownership or permissions of the destination OS.
set -eu
[ "$#" = 2 ] && [ -d "$1" ] && [ -d "$2" ] || {
  echo 'usage: apply-overlay.sh PUBLIC_OVERLAY EXISTING_TARGET_ROOT' >&2; exit 2;
}
[ "$(id -u)" = 0 ] || { echo 'root required for system overlay ownership' >&2; exit 1; }
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT HUP INT TERM
mkdir "$work/overlay"
cp -a "$1/." "$work/overlay/"
chown -hR 0:0 "$work/overlay"
find "$work/overlay" -type d -exec chmod 0755 {} +
# Retain executable bits and deliberately restrictive file modes; only remove
# group/world writes. Symlinks are not followed by either normalization step.
find "$work/overlay" -type f -exec chmod go-w {} +
tar --numeric-owner -C "$work/overlay" -cf "$work/overlay.tar" .
tar --same-owner --same-permissions -C "$2" -xf "$work/overlay.tar"
