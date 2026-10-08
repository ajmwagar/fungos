# Manual build and inspection runbook

## Prerequisites

Use a Debian-compatible host with `mmdebstrap`, `debian-archive-keyring`,
`tar`, `sha256sum`, and `shellcheck`.
Building `arm64` on another architecture also needs `qemu-user-static` and
`binfmt-support`. The build requires root (or a working rootless mmdebstrap
mode), outbound HTTPS, and enough free space for one root filesystem.

## Build

1. From the repository root, change to `base`.
2. Run `./scripts/check.sh` to validate declarations and shell code.
3. Run `sudo ./scripts/build.sh amd64` or `sudo ./scripts/build.sh arm64`.
4. Find the rootfs tarball, manifest, package list, and SHA-256 checksum in
   `out/`.
5. Run `sudo ./scripts/inspect.sh out/fungos-base-ARCH.tar`.

The build refuses unsupported architectures and unknown or duplicate
capabilities. Override `OUT_DIR` or `WORK_DIR` for CI scratch storage. Do not
override the snapshot for a release without reviewing and committing the new
lock values.

## Reproducibility check

Build twice from clean work directories with the same committed inputs, then:

```sh
sha256sum out-a/fungos-base-amd64.tar out-b/fungos-base-amd64.tar
diff -u out-a/fungos-base-amd64.packages out-b/fungos-base-amd64.packages
```

The digests should match. A mismatch means the snapshot changed unexpectedly,
the archive normalization regressed, or a build input was not declared.

## First boot

1. Add a machine-specific `/etc/fungos/first-contact.env` through provisioning.
   Never put credentials in the image.
2. Install exactly one executable adapter in
   `/usr/lib/fungos/first-contact.d/`. It receives the environment file path as
   its sole argument.
3. Add authorized SSH keys through provisioning. Root password login is
   disabled in the image.
4. Reboot or start `fungos-first-contact.service`; inspect it with
   `journalctl -u fungos-first-contact`.
5. Install an update verifier adapter under
   `/usr/lib/fungos/update-verifiers.d/`; invoke `fungos-verify-update` before
   applying any artifact.

## Recovery

If enrollment fails, inspect the journal, networkd state, the non-secret
environment file, and adapter permissions. The runner fails loudly when zero
or multiple adapters exist. Remove stale adapters rather than selecting one by
filesystem order.
