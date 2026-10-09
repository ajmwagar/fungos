#!/bin/sh
# Extend the existing shell image tooling; Debian d-i owns partitioning.
set -eu
installer_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
base_dir="$installer_dir/../base"
. "$base_dir/config/release.env"
[ "$DEBIAN_SUITE" = trixie ] || { echo 'installer requires the shared trixie baseline; refresh base/config/release.env before building' >&2; exit 1; }
[ "$#" = 2 ] || [ "$#" = 3 ] || { echo 'usage: build.sh VERIFIED_DEBIAN_ISO NEW_OUTPUT_ISO [VERIFIED_COMPONENT_BUNDLE]' >&2; exit 2; }
input=$(realpath "$1")
output=$(realpath -m "$2")
[ -f "$input" ] && [ ! -e "$output" ] || { echo 'input missing or output exists' >&2; exit 1; }
expected=658b28e209b578fe788ec5867deebae57b6aac5fce3692bbb116bab9c65568b3
[ "$(sha256sum "$input" | cut -d' ' -f1)" = "$expected" ] || { echo 'Debian 13.1.0 amd64 netinst digest mismatch' >&2; exit 1; }
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT HUP INT TERM
mkdir "$work/fungos"
cp -a "$base_dir/rootfs" "$work/fungos/rootfs"
[ ! -e "$work/fungos/rootfs/etc/fungos/first-contact.env" ] || { echo 'first-contact environment must not enter public ISO' >&2; exit 1; }
if find "$work/fungos/rootfs" \( -name 'ssh_host_*' -o -name '.mycelium' -o -name 'credentials.env' -o -name '.env' \) | grep -q .; then
  echo 'host identity or credential state found in public overlay' >&2
  exit 1
fi
source_commit=${SOURCE_COMMIT:-$(git -C "$installer_dir" rev-parse HEAD)}
[ "${#source_commit}" = 40 ] || { echo 'SOURCE_COMMIT must be a full source revision' >&2; exit 1; }
case "$source_commit" in *[!0-9a-f]*) echo 'SOURCE_COMMIT must be lowercase hexadecimal' >&2; exit 1 ;; esac
cp "$base_dir/config/release.env" "$installer_dir/apply.sh" "$work/fungos/"
runtime_components=absent
if [ "$#" = 3 ]; then
  component_tool=${FUNGOS_COMPONENT_TOOL:-"$installer_dir/../packages/target/release/component-bundle"}
  [ -x "$component_tool" ] || { echo 'build the Rust component-bundle verifier first' >&2; exit 1; }
  "$component_tool" verify "$3"
  (cd "$3" && sha256sum -c SHA256SUMS)
  # Only copy validated filenames, not unrelated files from the build directory.
  mkdir "$work/fungos/components"
  cp "$3/components.tsv" "$3/SHA256SUMS" "$work/fungos/components/"
  while IFS="$(printf '\t')" read -r name version architecture digest file source revision license ownership; do
    case "$name" in ''|'#'*) continue ;; esac
    [ "$architecture" = amd64 ] || { echo 'amd64 installer requires amd64 components' >&2; exit 1; }
    cp "$3/$file" "$work/fungos/components/$file"
  done < "$3/components.tsv"
  runtime_components=owner-debs
fi
"$base_dir/scripts/resolve-packages.sh" "$base_dir/profiles/headless-edge.capabilities" > "$work/fungos/packages"
cp "$installer_dir/interactive.preseed" "$work/fungos/interactive.preseed"
# Apply the common profile automatically, without answering disk/account prompts.
xorriso -osirrox on -indev "$input" -extract /isolinux/txt.cfg "$work/txt.cfg" -extract /boot/grub/grub.cfg "$work/grub.cfg" -end
sed 's@append @append preseed/file=/cdrom/fungos/interactive.preseed @' "$work/txt.cfg" > "$work/txt-new.cfg"
sed '/^[[:space:]]*linux[[:space:]]/s@ ---@ preseed/file=/cdrom/fungos/interactive.preseed ---@' "$work/grub.cfg" > "$work/grub-new.cfg"
xorriso -osirrox on -indev "$input" -extract /md5sum.txt "$work/md5sum-original.txt" -end
awk '$2 != "./isolinux/txt.cfg" && $2 != "./boot/grub/grub.cfg"' "$work/md5sum-original.txt" > "$work/md5sum.txt"
printf '%s  ./isolinux/txt.cfg\n' "$(md5sum "$work/txt-new.cfg" | cut -d' ' -f1)" >> "$work/md5sum.txt"
printf '%s  ./boot/grub/grub.cfg\n' "$(md5sum "$work/grub-new.cfg" | cut -d' ' -f1)" >> "$work/md5sum.txt"
(cd "$work" && find fungos -type f -exec md5sum {} +) | sed 's@  fungos/@  ./fungos/@' >> "$work/md5sum.txt"
# Replay Debian's BIOS/UEFI/hybrid boot layout. Public ISO contains no destructive
# confirmation answers; interactive partitioning remains Debian's responsibility.
xorriso -indev "$input" -outdev "$output" -boot_image any replay \
  -map "$work/fungos" /fungos -map "$work/txt-new.cfg" /isolinux/txt.cfg \
  -map "$work/grub-new.cfg" /boot/grub/grub.cfg -map "$work/md5sum.txt" /md5sum.txt -commit -end
sha256sum "$output" > "$output.sha256"
{
  echo "profile=headless-edge"
  echo "runtime_components=$runtime_components"
  if [ "$runtime_components" = owner-debs ]; then
    echo "component_manifest_sha256=$(sha256sum "$work/fungos/components/components.tsv" | cut -d' ' -f1)"
  fi
  echo "debian_suite=$DEBIAN_SUITE"
  echo "debian_snapshot=$DEBIAN_SNAPSHOT"
  echo "installer_upstream=debian-13.1.0-amd64-netinst.iso"
  echo "installer_upstream_sha256=$expected"
  echo "source_commit=$source_commit"
  echo "source_payload_sha256=$(tar -C "$work" -cf - fungos | sha256sum | cut -d' ' -f1)"
  echo "xorriso=$(xorriso -version 2>/dev/null | head -1)"
  echo "artifact_sha256=$(sha256sum "$output" | cut -d' ' -f1)"
} > "$output.manifest"
echo "built $output"
