# Debian 13 baseline refresh

The shared lock now selects `trixie`, cutoff `20261009T000000Z` for Debian,
updates and Debian Security. Installer and rootfs builds use that same lock;
the installer still begins from its separately hash-verified Debian 13.1 ISO
before converging installed packages. This is a new-image baseline, not an
in-place upgrade policy.

On 9 October 2026, Agora verified these snapshot `InRelease` files with
`gpgv` and `/usr/share/keyrings/debian-archive-keyring.gpg`. Good signatures
included Debian 13 archive/security keys; no trust or expiry bypass was used
for signature verification.

| Archive suite | Signed metadata date (UTC) | InRelease SHA-256 |
| --- | --- | --- |
| trixie (13.7) | 2026-09-12 07:55:41 | `0584fba32e13e0ab8285fb16c27adea1ec03a73669c18702821094fd6ca86675` |
| trixie-updates | 2026-10-08 20:20:09 | `0a19965c49878dc52ac41ba06ae6116f00cd0dc1b9bd7386b953e41e4d11c205` |
| trixie-security | 2026-10-08 23:02:22 | `4e3d5b0b8cee991cb99a0a49d423cfc67c218aa0f8afa4589d6a1c88e1bf5bf7` |

Updates/security metadata were valid until 15 October 2026. Immutable builds
disable APT time expiry, not signature or package hash validation. Snapshot
cutoffs may resolve to an earlier import, so actual signed metadata is retained
instead of treating a date string as security evidence.

Verification host tools: `debian-archive-keyring 2025.1ubuntu1`,
`mmdebstrap 1.5.7-3`. Keyring SHA-256:
`506b815cbb32d9b6066b4a2aa524071e071761e7e7f68c3ac74f3061ba852017`.
Raw verification inputs are retained on Agora in
`/home/ajmwagar/.cache/fungos-baseline-signatures.6BbXXP`.

The isolated amd64 headless-edge build completed in 70 seconds and filesystem
inspection passed. Retained build and inputs:
`/home/ajmwagar/.cache/fungos-baseline-refresh.PXJ0r5/out`.
Its 126-row installed binary/source inventory has SHA-256
`fbb0addada5fc687e9553e408de333d452cee3607c985cdb2fed1d1fb65a230c`;
the rootfs tar has SHA-256
`15899a0a315ccdafc455e4d79a8a029ec608dcf933bb7963068fa01d8225d368`.
All three release metadata files retained by that build independently passed
`gpgv` verification. Installed versions include OpenSSH
`1:10.0p1-7+deb13u4` and systemd `257.13-1~deb13u1`.

Builds now retain exact binary/source versions and authenticated release
metadata alongside the tarball. Those are audit and source-closure inputs,
not a complete source archive, CVE clearance or STIG certification. An amd64
headless-edge filesystem qualification does not prove ARM boot, installer
interaction, enrollment, applications or update/rollback behavior. Final ISO
and runtime qualification must use the refreshed bytes, not historical proof.

References: [Debian snapshot semantics](https://snapshot.debian.org/)
and [Debian trixie release information](https://www.debian.org/releases/trixie/).
