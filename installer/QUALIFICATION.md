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
