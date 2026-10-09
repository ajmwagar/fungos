# Installer qualification

An installer login prompt is an OS boot result, not an operational fungOS release
result. Qualification must identify the exact ISO digest, package inventory and
component manifest. Changing the Debian snapshot or component bundle invalidates
the prior final-image qualification.

## Required final-image evidence

| Gate | Evidence |
|---|---|
| Public interactive entry | Boot the unmodified ISO, select Install, complete the account and disk-confirmation flow on a new disposable disk. |
| BIOS and UEFI installation | Run `qemu-proof.sh` with each firmware mode; preserve input manifest, installer log and installed-disk-only boot log. |
| First boot | Verify generated machine ID and SSH host key, wired networking, failed units and expected component services. |
| Enrollment and SSH | Deliver a private short-lived test claim separately from the public ISO; authenticate using the isolated test authority, verify certificate-only SSH and rejection without a grant. |
| Signed update and rollback | Install genuine component packages, reject tampered/untrusted input, activate a signed update and verify rollback using the owner updater. |
| Reboot persistence | Boot from installed disk again; verify peer identity, SSH host key and installed component versions remain correct. |

Use only fresh harness-created disks. No physical disk, production identity,
private key, password or claim belongs in public media. Retain source/license
inputs for genuine component packages; fixtures cannot prove the runtime gates.
Secure Boot is separate from ordinary UEFI qualification.

## Preparatory public UEFI entry (2026-10-09)

The retained ISO from the earlier BIOS experiment has SHA-256
`c72c97966d464fa0299351121704ad753d7fa66a58f6dba8d5c1579b261e3c5d`.
It uses trixie snapshot `20261004T000000Z`, not the refreshed release baseline.
On Agora, QEMU with OVMF booted its real public UEFI menu. Selecting **Install**
reached the interactive language screen. No disk or network adapter was attached
for this menu-only check; it does not prove an interactive installation.

Raw QEMU framebuffer captures remain in
`/home/ajmwagar/.cache/fungos-uefi-qualification.4sfRDU/`:

| Capture | SHA-256 |
|---|---|
| `public-menu.ppm` | `995273a89248b1bcce5d805cc54d405a597fb3503e3b8757d5c2ee2322841619` |
| `public-interactive.ppm` | `28bf23c79e06344ad9910180c037d525f6427acdbca0b6930c3e3695212ab8bc` |

The public-menu QEMU was stopped afterward. The preexisting edge QEMU was not
changed. Final baseline/component enrollment and update gates remain open in
Marbles (`mycelium-v34j`, `mycelium-fungos-component-packaging`).

## Refreshed baseline install and reboot (2026-10-09)

Built the OS-only installer from committed source
`c1b1fc74bb8bd509abe16df758b819e565ca3878`, with explicit `SOURCE_COMMIT` for
the archived source tree. This candidate uses trixie snapshot
`20261009T000000Z`; it does **not** contain the pending Mycelium/Unibus owner
package bundle. It is not a v0.1.0 release candidate for operational use.

- ISO: `fungos-trixie-20261009-v2.iso`, SHA-256
  `ae1aabf257bc84880592c2cb213ff0c7c9051bfbf870d3944fd892eeb9fd0ffa`.
- Archived source SHA-256:
  `1281ca5672d357897bd70d7dd3b2c3958546ddec9d5d2ec10d87a14a951436e2`.
- Evidence root on Agora:
  `/home/ajmwagar/.cache/fungos-uefi-qualification.4sfRDU/`.

Both fresh disposable installations emitted `FUNGOS_INSTALL_COMPLETE` after
profile convergence and GRUB configuration. Separate disk-only BIOS and UEFI
boots reached `fungos login:` with kernel `6.12.111+deb13-amd64`. The UEFI log
records OVMF starting `\EFI\debian\shimx64.efi` from the installed GPT ESP.
BIOS harness exited zero. The copied pre-fix UEFI harness exited one because
GRUB's quiet default suppresses the kernel EFI banner; the corrected check in
`600235d` was independently applied to its retained logs and passed. This is
not a claim that the pre-fix harness exited successfully.

| Evidence | SHA-256 |
|---|---|
| `fresh-bios-proof/install.log` | `707f49b93327254107d8c1894a10acfa2db6c019471afb2423e47909e1b4ec5d` |
| `fresh-bios-proof/boot.log` | `3fd941d2648d566a50c6d9859942b2ae52457a3c16105c3bbba9e607d91a62a0` |
| `fresh-uefi-proof/install.log` | `508be188e6987454d0957bf6b33cf24e698e826880e88272154f279450112660` |
| `fresh-uefi-proof/boot.log` | `d38bd6a8577cfe00d9d93d064d02c41a5178c4fb21f335a4152137329770b50a` |
| `fresh-uefi-proof/reboot-persistence.log` | `c08f7e8b14ff9596f9dbc64752ddbf40e6c0c884c910c148be4fe9ecadcb575c` |

A further installed-disk-only UEFI boot reached login, then shut down using its
private QEMU monitor's ACPI power button. Read-only disk inspections before and
after that reboot found unchanged machine-ID and ED25519 host-public-key hashes,
and unchanged 214-row binary/source package inventory. Inventory SHA-256 is
`8c8ba0e00c2b50596b45d7c3b0f56feeca379ab75569f3f22ea69aae1986b40d`.
The shared SSH/networkd units are enabled; the generated host public key exists;
no first-contact claim environment or installer payload remains. Read-only loop
devices were detached and all qualification QEMUs stopped afterward.

The Debian `systemd-ssh-generator` AF_VSOCK CID warning remains visible. These
checks do not establish whole-system unit health, a completed public interactive
installation, production enrollment, personal certificate SSH, Secure Boot or
real component update/rollback. Those gates remain open.
