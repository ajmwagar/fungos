# fungOS headless installer (amd64, first slice)

Final-image gates and digest-bound preparatory evidence are in
[QUALIFICATION.md](QUALIFICATION.md).

An explicit Mycelium-only qualification payload is supported by the shared
[component gate](../packages/COMPONENTS.md). Supply its verified bundle plus
the fourth argument `mycelium-qualification`. Normal component builds still
require Mycelium **and** Unibus. The narrow mode enables only the Mycelium unit
and labels its manifest accordingly; it does not claim a full headless-edge
runtime release. Private enrollment and person access remain post-install
qualification, never public ISO contents.

This is a Debian 13 installer delivery wrapper, not a second operating-system
package pipeline. It resolves `base/profiles/headless-edge.capabilities`, applies
the common `base/rootfs` configuration, and converges Debian packages to the
signed snapshot in `base/config/release.env`. Debian Installer owns partitioning
and bootloader installation. Its required installer/kernel packages mean this
is not a byte-identical copy of the minbase rootfs tar. This first slice requires network access during
the late package-convergence step; it is not an offline installer.

The public ISO retains interactive account/disk selection and erasure
confirmation. It contains **no** administrator password, SSH private key,
enrollment claim, certificate or unattended partitioning approval. The QEMU
preseed is separate and never included in the public ISO. Managed physical
installation authorization, encryption/Tang/Clevis and Canvas are later work.

## Build and test on Linux

Use Agora (or another isolated Linux build host), not Neo, for image work.
Install existing third-party `xorriso`, `cpio`, QEMU and standard Debian tools.
The builder verifies this exact upstream input before remastering:

```sh
curl -fSL https://cdimage.debian.org/cdimage/archive/13.1.0/amd64/iso-cd/debian-13.1.0-amd64-netinst.iso -o debian.iso
sh installer/check.sh
sh installer/build.sh debian.iso fungos-headless-edge-amd64.iso
sudo sh installer/qemu-proof.sh fungos-headless-edge-amd64.iso ./new-bios-proof
sudo sh installer/qemu-proof.sh fungos-headless-edge-amd64.iso ./new-uefi-proof uefi
```

The optional UEFI proof requires Debian/Ubuntu OVMF (`ovmf`). Its code is read-only;
the harness copies the variable-store template into the fresh proof directory,
then reuses that private NVRAM for the installed-disk reboot. Override the template
paths with `FUNGOS_OVMF_CODE` and `FUNGOS_OVMF_VARS` if needed. Secure Boot is not
enabled or proven by this mode. The 8 GiB UEFI test uses Debian's `small_disk`
partition recipe: its `atomic` recipe requires more space for root, ESP and swap.
The ISO preserves Debian's hybrid BIOS/UEFI boot structures; preserving those
structures alone does not establish successful installation. For the text installer choose **Install**.
Other installer entry variants are not yet validated.

The harness refuses an existing output directory, creates its own blank 8 GiB
qcow2 disk and exposes only that writable disk. Before partitioning, the guest
guard requires one `vda` disk with the exact harness serial and size; it rejects
an absent, unexpected or additional disk. There is no option to supply a real
host block device. It first boots Debian's installer kernel/initrd with the
public ISO attached. Then it starts QEMU again with **only the installed disk**:
no ISO, external kernel or initrd. Serial logs and the disposable disk remain
in the requested proof directory for inspection. Test accounts are locked;
reaching a login prompt does not prove SSH enrollment or signed updates.

The first-contact and SSH-host-key hooks are common with the rootfs build.
Machine ID and host keys are reset during installation and generated on first
boot. Enrollment still needs an existing authorized claim delivery adapter;
this ISO does not introduce a new claim protocol or distribute secrets over
preseed, kernel arguments, TFTP or public HTTP. Mycelium/Unibus executable
releases are not embedded by the two-argument baseline build. The optional third
argument includes a [verified owner-component bundle](../packages/COMPONENTS.md).
Only final operational qualification can establish release readiness; a baseline
ISO explicitly records `runtime_components=absent` in its manifest.

## Release evidence

The builder writes `.sha256` and `.manifest`, including the upstream installer
digest, Debian snapshot and source revision/payload digest. Archive the exact
source tree used for the build alongside the result; the revision alone is not
sufficient when testing uncommitted modifications. An installed target retains
`/usr/share/fungos/installer/installed.tsv` with exact binary/source package
versions and architecture. These inputs support corresponding-source assembly;
they are not themselves the corresponding-source archive. Do not publish a
release until source/license inventory and boot gates are complete.

Manual recovery: inspect `install.log`; rerun with a **new** proof directory.
Never reuse a partially installed disk or substitute a live `/dev/...` target.
If snapshot access fails, the late installer step fails rather than silently
switching to rolling Debian mirrors.

Upstream references: [Debian preseeding](https://www.debian.org/releases/trixie/amd64/apbs02.en.html)
and [supported installer answers](https://www.debian.org/releases/trixie/amd64/apbs04.en.html).

## First BIOS proof (2026-10-06)

The ISO produced from tooling commit `f3c1a0e` installed unattended on Agora's
disposable 8 GiB QEMU disk. A separate QEMU invocation booted only that disk:
GRUB loaded installed Debian kernel `6.12.111+deb13-amd64` and its initramfs,
then reached `Debian GNU/Linux 13 fungos ttyS0` / `fungos login:`. Installer media
file checksums also passed after remastering.

- ISO size: `820314112` bytes.
- SHA-256: `c72c97966d464fa0299351121704ad753d7fa66a58f6dba8d5c1579b261e3c5d`.
- Agora artifact: `/home/ajmwagar/.cache/fungos-installer.HRoBKE/fungos-headless-edge-amd64-v3.iso`.
- Serial evidence: `bios-proof9/install.log` and `bios-proof9/boot.log` in that directory.

Read-only installed-disk inspection verified a fresh machine ID, generated
ED25519 SSH host key, enabled networkd/first-contact hooks, absent claim
environment and removal of the installer payload. The exact installed
binary/source inventory has 206 rows and SHA-256
`227299fe29b2c07191848e3649629825db06f28180b3fcde7dcc712122f5c069`.
The harness exited successfully and its QEMU processes were stopped. These
offline checks do not establish running service health or SSH authorization.

This proves unattended installation and native installed-disk BIOS boot, not
public ISO menu interaction, UEFI/Secure Boot, SSH enrollment, runtime updates,
or whole-system health. The first boot also reports a Debian
`systemd-ssh-generator` AF_VSOCK CID warning in this QEMU configuration; that
warning remains a follow-up before calling the image operationally clean.
The later public-overlay credential guard changes do not change installed
profile contents; their rejection test is separate from this recorded ISO.
