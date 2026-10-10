#!/bin/sh
set -eu

base_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
# shellcheck source=/dev/null
. "$base_dir/config/release.env"

arch=${1:-}
profile=${2:-base}
case "$arch" in amd64|arm64) ;; *) echo "usage: $0 {amd64|arm64} [base|edge|headless-edge|cloud]" >&2; exit 2;; esac
case "$profile" in base|edge|headless-edge|cloud) ;; *) echo "unsupported profile: $profile" >&2; exit 2;; esac

command -v mmdebstrap >/dev/null || { echo "mmdebstrap is required" >&2; exit 1; }
debian_keyring=/usr/share/keyrings/debian-archive-keyring.gpg
[ -r "$debian_keyring" ] || {
  echo "Debian archive keyring is required at $debian_keyring" >&2
  exit 1
}
out_dir=${OUT_DIR:-"$base_dir/out"}
work_dir=${WORK_DIR:-"$base_dir/.work/$profile/$arch"}
rootfs="$work_dir/rootfs"
artifact="$out_dir/fungos-$profile-$arch.tar"
package_file="$out_dir/fungos-$profile-$arch.packages"
manifest="$out_dir/fungos-$profile-$arch.manifest"
inventory="$out_dir/fungos-$profile-$arch.installed.tsv"
release_inputs="$out_dir/fungos-$profile-$arch.release-inputs"

mkdir -p "$out_dir" "$work_dir"
[ ! -e "$rootfs" ] || { echo "refusing non-clean work directory: $rootfs" >&2; exit 1; }
overlay=${CAPABILITY_OVERLAY:-}
"$base_dir/scripts/resolve-packages.sh" "$base_dir/profiles/$profile.capabilities" "$overlay" > "$package_file"
packages=$(paste -sd, "$package_file")
mirror="deb [check-valid-until=no] https://snapshot.debian.org/archive/debian/$DEBIAN_SNAPSHOT $DEBIAN_SUITE main"
updates_mirror="deb [check-valid-until=no] https://snapshot.debian.org/archive/debian/$DEBIAN_SNAPSHOT $DEBIAN_SUITE-updates main"
security_mirror="deb [check-valid-until=no] https://snapshot.debian.org/archive/debian-security/$DEBIAN_SNAPSHOT $DEBIAN_SUITE-security main"

mmdebstrap \
  --skip=cleanup/apt/lists \
  --variant=minbase \
  --keyring="$debian_keyring" \
  --architectures="$arch" \
  --include="$packages" \
  --aptopt='Acquire::Languages "none"' \
  --aptopt='APT::Install-Recommends "false"' \
  --aptopt='APT::Install-Suggests "false"' \
  --dpkgopt='path-exclude=/usr/share/doc/*' \
  --dpkgopt='path-include=/usr/share/doc/*/copyright' \
  --dpkgopt='path-exclude=/usr/share/man/*' \
  --dpkgopt='path-exclude=/usr/share/locale/*' \
  "$DEBIAN_SUITE" "$rootfs" "$mirror" "$updates_mirror" "$security_mirror"

# APT validates signed release metadata and package hashes. Keep its exact
# authenticated inputs alongside source/version inventory before sanitizing.
chroot "$rootfs" dpkg-query -W -f='${binary:Package}\t${Version}\t${source:Package}\t${source:Version}\t${Architecture}\n' \
  | LC_ALL=C sort > "$inventory"
[ -s "$inventory" ] || { echo 'installed package inventory is empty' >&2; exit 1; }
mkdir -p "$release_inputs"
find "$rootfs/var/lib/apt/lists" -maxdepth 1 -type f -name '*InRelease' -exec cp {} "$release_inputs/" \;
[ "$(find "$release_inputs" -type f -name '*InRelease' | wc -l)" -eq 3 ] || { echo 'expected three authenticated archive releases' >&2; exit 1; }
(cd "$release_inputs" && sha256sum *InRelease) > "$release_inputs/SHA256SUMS"

sh "$base_dir/scripts/apply-overlay.sh" "$base_dir/rootfs" "$rootfs"
chmod 0755 "$rootfs/usr/libexec/fungos-first-contact" "$rootfs/usr/libexec/fungos-verify-update"
mkdir -p \
  "$rootfs/etc/systemd/system/multi-user.target.wants" \
  "$rootfs/etc/systemd/system/network-online.target.wants" \
  "$rootfs/usr/lib/fungos/first-contact.d" \
  "$rootfs/usr/lib/fungos/update-verifiers.d"
ln -sfn /run/systemd/resolve/stub-resolv.conf "$rootfs/etc/resolv.conf"
ln -sfn /usr/lib/systemd/system/systemd-networkd.service "$rootfs/etc/systemd/system/multi-user.target.wants/systemd-networkd.service"
ln -sfn /usr/lib/systemd/system/systemd-resolved.service "$rootfs/etc/systemd/system/multi-user.target.wants/systemd-resolved.service"
ln -sfn /usr/lib/systemd/system/systemd-networkd-wait-online.service "$rootfs/etc/systemd/system/network-online.target.wants/systemd-networkd-wait-online.service"
ln -sfn /usr/lib/systemd/system/ssh.service "$rootfs/etc/systemd/system/multi-user.target.wants/ssh.service"
ln -sfn /usr/lib/systemd/system/fungos-first-contact.service "$rootfs/etc/systemd/system/multi-user.target.wants/fungos-first-contact.service"

# Remove host-specific and time-varying state. systemd and ssh recreate these on
# first boot; package indexes are release inputs, not runtime image content.
find "$rootfs/etc/ssh" -maxdepth 1 -type f -name 'ssh_host_*' -delete
find "$rootfs/var/log" -type f -exec truncate -s 0 {} +
find "$rootfs/var/lib/apt/lists" -mindepth 1 -delete
: > "$rootfs/etc/machine-id"

find "$rootfs" -xdev -print0 | xargs -0 touch --no-dereference --date="@$SOURCE_DATE_EPOCH"
tar --sort=name --format=posix --numeric-owner --owner=0 --group=0 \
  --mtime="@$SOURCE_DATE_EPOCH" --pax-option=delete=atime,delete=ctime \
  -C "$rootfs" -cf "$artifact" .

{
  echo "architecture=$arch"
  echo "debian_suite=$DEBIAN_SUITE"
  echo "debian_snapshot=$DEBIAN_SNAPSHOT"
  echo "source_date_epoch=$SOURCE_DATE_EPOCH"
  echo "profile=$profile"
  echo "artifact=$(basename "$artifact")"
  echo "artifact_sha256=$(sha256sum "$artifact" | cut -d' ' -f1)"
  echo "packages_sha256=$(sha256sum "$package_file" | cut -d' ' -f1)"
  echo "installed_sha256=$(sha256sum "$inventory" | cut -d' ' -f1)"
  echo "release_inputs_sha256=$(sha256sum "$release_inputs/SHA256SUMS" | cut -d' ' -f1)"
  echo "archive_suites=$DEBIAN_SUITE,$DEBIAN_SUITE-updates,$DEBIAN_SUITE-security"
} > "$manifest"
(cd "$out_dir" && sha256sum "$(basename "$artifact")") > "$artifact.sha256"
echo "built $artifact"
