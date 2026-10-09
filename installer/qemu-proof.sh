#!/bin/sh
# Never accepts a caller's existing disk. Only a newly created tempfile is exposed.
set -eu
[ "$#" = 2 ] || [ "$#" = 3 ] || { echo 'usage: qemu-proof.sh FUNGOS_ISO NEW_PROOF_DIRECTORY [bios|uefi]' >&2; exit 2; }
firmware=${3:-bios}
case "$firmware" in
 bios) ;;
 uefi)
  ovmf_code=${FUNGOS_OVMF_CODE:-/usr/share/OVMF/OVMF_CODE_4M.fd}
  ovmf_vars=${FUNGOS_OVMF_VARS:-/usr/share/OVMF/OVMF_VARS_4M.fd}
  [ -f "$ovmf_code" ] && [ -f "$ovmf_vars" ] || { echo 'OVMF code/variables missing' >&2; exit 1; }
  ;;
 *) echo 'firmware must be bios or uefi' >&2; exit 2 ;;
esac
[ "$(id -u)" = 0 ] || { echo 'root required to preserve Debian initrd device nodes' >&2; exit 1; }
iso=$(realpath "$1")
proof=$(realpath -m "$2")
[ -f "$iso" ] && [ ! -e "$proof" ] || { echo 'ISO missing or proof directory exists' >&2; exit 1; }
mkdir -m 0700 "$proof"
set --
if [ "$firmware" = uefi ]; then
 # Never mutate the shared template. Both invocations use the same private NVRAM.
 cp "$ovmf_vars" "$proof/OVMF_VARS.fd"
 set -- -machine q35 -drive "if=pflash,format=raw,readonly=on,file=$ovmf_code" -drive "if=pflash,format=raw,file=$proof/OVMF_VARS.fd"
fi
{
  echo "firmware=$firmware"
  if [ "$firmware" = uefi ]; then
   echo "ovmf_code_sha256=$(sha256sum "$ovmf_code" | cut -d' ' -f1)"
   echo "ovmf_vars_template_sha256=$(sha256sum "$ovmf_vars" | cut -d' ' -f1)"
  fi
  echo "iso_sha256=$(sha256sum "$iso" | cut -d' ' -f1)"
  echo "qemu=$(qemu-system-x86_64 --version | head -1)"
  echo 'disk_bytes=8589934592'
  echo 'disk_serial=FUNGOS_DISPOSABLE'
  if [ "$firmware" = uefi ]; then echo 'partition_recipe=small_disk'; else echo 'partition_recipe=atomic'; fi
} > "$proof/inputs.manifest"
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
qemu_pid=''
cleanup() { [ -z "$qemu_pid" ] || kill "$qemu_pid" 2>/dev/null || true; }
trap cleanup EXIT HUP INT TERM
xorriso -osirrox on -indev "$iso" -extract /install.amd/vmlinuz "$proof/vmlinuz" -extract /install.amd/initrd.gz "$proof/initrd.gz" -end
chmod u+w "$proof/initrd.gz"
mkdir "$proof/inject"
(cd "$proof/inject" && gzip -dc "$proof/initrd.gz" | cpio --quiet -id)
if [ "$firmware" = uefi ]; then
 # Debian's EFI atomic recipe needs more than 8 GiB (8 GB root + ESP + swap).
 # Use its existing small-disk recipe, keeping the exact safety guard unchanged.
 sed 's/partman-auto\/choose_recipe select atomic/partman-auto\/choose_recipe select small_disk/' \
  "$script_dir/test.preseed" > "$proof/inject/preseed.cfg"
else
 cp "$script_dir/test.preseed" "$proof/inject/preseed.cfg"
fi
cp "$script_dir/guard-disk.sh" "$proof/inject/guard-disk"
chmod 0755 "$proof/inject/guard-disk"
(cd "$proof/inject" && find . -print | cpio --quiet -H newc -o | gzip -n) > "$proof/initrd.gz"
qemu-img create -f qcow2 "$proof/disposable.qcow2" 8G
timeout 2400 qemu-system-x86_64 "$@" -enable-kvm -m 2048 -smp 2 -nographic -no-reboot \
 -kernel "$proof/vmlinuz" -initrd "$proof/initrd.gz" \
 -append 'auto=true priority=critical preseed/file=/preseed.cfg console=ttyS0,115200n8 DEBIAN_FRONTEND=text' \
 -drive "file=$proof/disposable.qcow2,format=qcow2,if=none,id=disk0" -device virtio-blk-pci,drive=disk0,serial=FUNGOS_DISPOSABLE \
 -cdrom "$iso" -netdev user,id=n0 -device virtio-net-pci,netdev=n0 > "$proof/install.log" 2>&1 &
qemu_pid=$!
wait "$qemu_pid"
qemu_pid=''
# Boot only installed disk: no kernel/initrd/ISO on this second invocation.
timeout 240 qemu-system-x86_64 "$@" -enable-kvm -m 1024 -smp 2 -nographic -no-reboot \
 -drive "file=$proof/disposable.qcow2,format=qcow2,if=none,id=disk0" -device virtio-blk-pci,drive=disk0,serial=FUNGOS_DISPOSABLE \
 -netdev user,id=n0 -device virtio-net-pci,netdev=n0 > "$proof/boot.log" 2>&1 &
qemu_pid=$!
wait "$qemu_pid" || [ "$?" = 124 ]
qemu_pid=''
grep -q 'fungos login:' "$proof/boot.log"
if [ "$firmware" = uefi ]; then
 grep -q 'efi: EFI v' "$proof/install.log"
 grep -q 'efi: EFI v' "$proof/boot.log"
fi
echo "installed-disk $firmware boot reached login; evidence: $proof"
