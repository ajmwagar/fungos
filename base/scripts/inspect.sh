#!/bin/sh
set -eu

artifact=${1:-}
[ -f "$artifact" ] || { echo "usage: $0 ROOTFS.tar" >&2; exit 2; }
for path in \
  ./usr/lib/systemd/system/fungos-first-contact.service \
  ./usr/libexec/fungos-first-contact \
  ./usr/libexec/fungos-verify-update \
  ./usr/lib/systemd/system/fungos-ssh-host-keys.service \
  ./etc/systemd/system/ssh.service.d/10-fungos-host-keys.conf \
  ./etc/systemd/network/20-wired.network \
  ./etc/ssh/sshd_config.d/10-fungos.conf; do
  tar -tf "$artifact" | grep -Fx "$path" >/dev/null || { echo "missing from image: $path" >&2; exit 1; }
done
echo "required fungOS base files are present"
