#!/bin/sh
# Never accepts a caller's existing disk. Only a newly created tempfile is exposed.
set -eu
[ "$(id -u)" = 0 ] || { echo 'root required to preserve Debian initrd device nodes' >&2; exit 1; }
[ "$#" = 2 ] || { echo 'usage: qemu-proof.sh FUNGOS_ISO NEW_PROOF_DIRECTORY' >&2; exit 2; }
iso=$(realpath "$1")
proof=$(realpath -m "$2")
[ -f "$iso" ] && [ ! -e "$proof" ] || { echo 'ISO missing or proof directory exists' >&2; exit 1; }
mkdir -m 0700 "$proof"
{
  echo "firmware=BIOS"
  echo "iso_sha256=$(sha256sum "$iso" | cut -d' ' -f1)"
  echo "qemu=$(qemu-system-x86_64 --version | head -1)"
  echo 'disk_bytes=8589934592'
  echo 'disk_serial=FUNGOS_DISPOSABLE'
} > "$proof/inputs.manifest"
script_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
qemu_pid=''
cleanup() { [ -z "$qemu_pid" ] || kill "$qemu_pid" 2>/dev/null || true; }
trap cleanup EXIT HUP INT TERM
xorriso -osirrox on -indev "$iso" -extract /install.amd/vmlinuz "$proof/vmlinuz" -extract /install.amd/initrd.gz "$proof/initrd.gz" -end
chmod u+w "$proof/initrd.gz"
mkdir "$proof/inject"
(cd "$proof/inject" && gzip -dc "$proof/initrd.gz" | cpio --quiet -id)
cp "$script_dir/test.preseed" "$proof/inject/preseed.cfg"
cp "$script_dir/guard-disk.sh" "$proof/inject/guard-disk"
chmod 0755 "$proof/inject/guard-disk"
(cd "$proof/inject" && find . -print | cpio --quiet -H newc -o | gzip -n) > "$proof/initrd.gz"
qemu-img create -f qcow2 "$proof/disposable.qcow2" 8G
timeout 2400 qemu-system-x86_64 -enable-kvm -m 2048 -smp 2 -nographic -no-reboot \
 -kernel "$proof/vmlinuz" -initrd "$proof/initrd.gz" \
 -append 'auto=true priority=critical preseed/file=/preseed.cfg console=ttyS0,115200n8 DEBIAN_FRONTEND=text' \
 -drive "file=$proof/disposable.qcow2,format=qcow2,if=none,id=disk0" -device virtio-blk-pci,drive=disk0,serial=FUNGOS_DISPOSABLE \
 -cdrom "$iso" -netdev user,id=n0 -device virtio-net-pci,netdev=n0 > "$proof/install.log" 2>&1 &
qemu_pid=$!
wait "$qemu_pid"
qemu_pid=''
# Boot only installed disk: no kernel/initrd/ISO on this second invocation.
timeout 240 qemu-system-x86_64 -enable-kvm -m 1024 -smp 2 -nographic -no-reboot \
 -drive "file=$proof/disposable.qcow2,format=qcow2,if=none,id=disk0" -device virtio-blk-pci,drive=disk0,serial=FUNGOS_DISPOSABLE \
 -netdev user,id=n0 -device virtio-net-pci,netdev=n0 > "$proof/boot.log" 2>&1 &
qemu_pid=$!
wait "$qemu_pid" || [ "$?" = 124 ]
qemu_pid=''
grep -q 'fungos login:' "$proof/boot.log"
echo "installed-disk BIOS boot reached login; evidence: $proof"
