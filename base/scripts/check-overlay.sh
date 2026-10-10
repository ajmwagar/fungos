#!/bin/sh
set -eu
[ "$(id -u)" = 0 ] || { echo 'root required for overlay ownership regression' >&2; exit 1; }
dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT HUP INT TERM
mkdir -p "$work/source/etc/ssh" "$work/source/usr/libexec" "$work/target/private"
printf 'public config\n' > "$work/source/etc/ssh/config"
printf '#!/bin/sh\nexit 0\n' > "$work/source/usr/libexec/hook"
chmod 0775 "$work/source" "$work/source/etc" "$work/source/etc/ssh" "$work/source/usr" "$work/source/usr/libexec" "$work/source/usr/libexec/hook"
chmod 0664 "$work/source/etc/ssh/config"
chown -R 1000:1000 "$work/source"
printf 'private stays private\n' > "$work/target/private/key"
chmod 0700 "$work/target/private"
chmod 0600 "$work/target/private/key"
chown 1234:1234 "$work/target/private/key"
ln -s /outside-unrelated "$work/source/etc/link"
sh "$dir/apply-overlay.sh" "$work/source" "$work/target"
for path in "$work/target" "$work/target/etc" "$work/target/etc/ssh"; do
  [ "$(stat -c '%u:%g:%a' "$path")" = 0:0:755 ] || { echo "unsafe system ancestor: $path" >&2; exit 1; }
done
[ "$(stat -c '%u:%g:%a' "$work/target/etc/ssh/config")" = 0:0:644 ]
[ "$(stat -c '%u:%g:%a' "$work/target/usr/libexec/hook")" = 0:0:755 ]
[ "$(stat -c '%u:%g:%a' "$work/target/private/key")" = 1234:1234:600 ]
[ "$(stat -c '%a' "$work/target/private")" = 700 ]
[ "$(cat "$work/target/private/key")" = 'private stays private' ]
[ "$(readlink "$work/target/etc/link")" = /outside-unrelated ]
echo 'nonroot-builder overlay normalized; unrelated private target preserved'
