#!/bin/sh
set -eu

base_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
profile=${1:-"$base_dir/profiles/base.capabilities"}

[ -f "$profile" ] || { echo "profile not found: $profile" >&2; exit 1; }

tmp_dir=$(mktemp -d)
trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM
capabilities="$tmp_dir/capabilities"
sed -e 's/[[:space:]]*#.*$//' -e '/^[[:space:]]*$/d' "$profile" > "$capabilities"
if [ -n "${2:-}" ]; then
  [ -f "$2" ] || { echo "capability overlay not found: $2" >&2; exit 1; }
  sed -e 's/[[:space:]]*#.*$//' -e '/^[[:space:]]*$/d' "$2" >> "$capabilities"
fi

[ -s "$capabilities" ] || { echo "profile has no capabilities" >&2; exit 1; }
if [ "$(sort "$capabilities" | uniq -d | wc -l | tr -d ' ')" -ne 0 ]; then
  echo "profile contains duplicate capabilities" >&2
  exit 1
fi

packages="$tmp_dir/packages"
: > "$packages"
while IFS= read -r capability; do
  # Composition reuses the base declaration; no copied base package list.
  case "$capability" in
    @base)
      "$base_dir/scripts/resolve-packages.sh" "$base_dir/profiles/base.capabilities" >> "$packages"
      continue
      ;;
  esac
  case "$capability" in *[!a-z0-9-]*|'') echo "invalid capability: $capability" >&2; exit 1;; esac
  declaration="$base_dir/capabilities/$capability.packages"
  [ -f "$declaration" ] || { echo "unknown capability: $capability" >&2; exit 1; }
  sed -e 's/[[:space:]]*#.*$//' -e '/^[[:space:]]*$/d' "$declaration" >> "$packages"
done < "$capabilities"

LC_ALL=C sort -u "$packages"
