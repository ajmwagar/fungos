#!/bin/sh
set -eu

base_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
# shellcheck source=/dev/null
. "$base_dir/config/release.env"
packages=$($base_dir/scripts/resolve-packages.sh)
[ -n "$packages" ] || { echo "resolved package set is empty" >&2; exit 1; }
[ "$packages" = "$(printf '%s\n' "$packages" | LC_ALL=C sort -u)" ] || {
  echo "package resolution is not deterministic" >&2; exit 1;
}

for arch in amd64 arm64; do
  case "$arch" in amd64|arm64) :;; *) exit 1;; esac
done

case "$DEBIAN_SUITE" in *[!a-z0-9-]*|'') echo "invalid Debian suite" >&2; exit 1;; esac
case "$DEBIAN_SNAPSHOT" in [0-9][0-9][0-9][0-9][0-9][0-9][0-9][0-9]T[0-9][0-9][0-9][0-9][0-9][0-9]Z) :;;
  *) echo "invalid Debian snapshot timestamp" >&2; exit 1;;
esac
case "$SOURCE_DATE_EPOCH" in *[!0-9]*|'') echo "invalid SOURCE_DATE_EPOCH" >&2; exit 1;; esac

while IFS= read -r package; do
  case "$package" in *[!a-z0-9+.-]*|'') echo "invalid package name: $package" >&2; exit 1;; esac
done <<EOF
$packages
EOF

for required in ca-certificates systemd systemd-sysv iproute2 kmod systemd-resolved udev openssh-server; do
  printf '%s\n' "$packages" | grep -Fx "$required" >/dev/null || {
    echo "missing required package: $required" >&2; exit 1;
  }
done

headless=$("$base_dir/scripts/resolve-packages.sh" "$base_dir/profiles/headless-edge.capabilities")
cloud=$("$base_dir/scripts/resolve-packages.sh" "$base_dir/profiles/cloud.capabilities")
for required in cpio curl dnsmasq-base dnsmasq-utils e2fsprogs iptables procps sudo; do
  printf '%s\n' "$cloud" | grep -Fx "$required" >/dev/null || {
    echo "cloud missing workload runtime: $required" >&2; exit 1;
  }
done
for package in $packages; do
  printf '%s\n' "$cloud" | grep -Fx "$package" >/dev/null || {
    echo "cloud failed to inherit base package: $package" >&2; exit 1;
  }
done
if printf '%s\n' "$cloud" | grep -E '^(libgbm1|libwayland-server0|mesa-vulkan-drivers|weston|xserver-xorg-core)$' >/dev/null; then
  echo "cloud includes display dependencies" >&2; exit 1
fi
edge=$("$base_dir/scripts/resolve-packages.sh" "$base_dir/profiles/edge.capabilities")
tooling=$("$base_dir/scripts/resolve-packages.sh" "$base_dir/profiles/edge.capabilities" "$base_dir/../tooling/shroudoci.capabilities")
for required in cpio skopeo squashfs-tools sudo umoci; do
  printf '%s\n' "$tooling" | grep -Fx "$required" >/dev/null || {
    echo "tooling missing converter dependency: $required" >&2; exit 1;
  }
done
for unchanged in "$packages" "$edge" "$headless"; do
  if printf '%s\n' "$unchanged" | grep -E '^(skopeo|umoci|squashfs-tools)$' >/dev/null; then
    echo "conversion tooling leaked into a default profile" >&2; exit 1
  fi
done
for inherited in $edge; do
  printf '%s\n' "$tooling" | grep -Fx "$inherited" >/dev/null || exit 1
done
if "$base_dir/scripts/resolve-packages.sh" "$base_dir/profiles/base.capabilities" "$base_dir/../tooling/does-not-exist" >/dev/null 2>&1; then
  echo "missing overlay was silently accepted" >&2; exit 1
fi
for forbidden in weston xserver-xorg-core sway; do
  if printf '%s\n' "$edge" "$headless" | grep -Fx "$forbidden" >/dev/null; then
    echo "unexpected external compositor: $forbidden" >&2; exit 1
  fi
done
if printf '%s\n' "$headless" | grep -E '^(mesa-vulkan-drivers|libwayland-server0|libgbm1)$' >/dev/null; then
  echo "headless-edge includes display dependencies" >&2; exit 1
fi
for required in libgbm1 libwayland-server0 mesa-vulkan-drivers; do
  printf '%s\n' "$edge" | grep -Fx "$required" >/dev/null || {
    echo "edge missing compositor runtime: $required" >&2; exit 1;
  }
done

if command -v shellcheck >/dev/null; then
  shellcheck "$base_dir"/scripts/*.sh "$base_dir"/rootfs/usr/libexec/fungos-*
else
  echo "warning: shellcheck not installed; syntax checks only" >&2
  for script in "$base_dir"/scripts/*.sh "$base_dir"/rootfs/usr/libexec/fungos-*; do sh -n "$script"; done
fi
echo "fungOS base declarations validated"
