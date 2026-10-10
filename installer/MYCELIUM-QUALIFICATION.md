# Mycelium-only installer qualification

On 10 October 2026, clean committed fungOS source
`5242f9c58abbe791e9ee7555f914d7e7de8d89d6` produced the amd64 installer ISO
SHA-256 `0ecdabec63266522989aceba6d980232d6ee109713d2313c9e45b116eaa99f50`.
The shared baseline is trixie `20261009T000000Z`. All remastered media file
checksums passed. The ISO includes no enrollment claim, private authority,
peer key or machine-specific credential overlay.

Its purpose, `mycelium-only-qualification`, is retained in the manifest and
installed `/usr/share/fungos/components/runtime-components`. This is **not** a
full headless-edge runtime release. Normal component verification still
requires Mycelium and Unibus; explicit narrow commands require exactly
Mycelium. The actual single-package bundle passed narrow verification and
failed full verification. Ten Rust component tests and installer safety
checks passed. No substitute package or license change was used.

## Actual installation and first contact

The ISO embeds genuine owner package `mycelium_0.1.0_amd64.deb`, SHA-256
`67db12ef7ea04b34e9a7ea9125d5bd563ee22f378244db00f41b82cf1c67a556`,
from Mycelium source `138feeb298174b9158e03bf2c76692572ce3e7df`.
The guarded harness created its own blank 8 GiB QEMU disk and performed actual
UEFI installation and package convergence, emitting `FUNGOS_INSTALL_COMPLETE`.
A separate disk-only native UEFI boot reached `fungos login:` with installed
kernel `6.12.111+deb13-amd64`. The harness exited zero and stopped its QEMU.

Private `--preinstalled` helpers from fungOS commit `50588c6` then converted
an isolated clone to raw format and injected only qualification fixtures.
They did not replace the installed executable or publish the clone. Two
native UEFI boots used the same raw disk and firmware variables, with no NIC,
ISO, external kernel or initrd. Both exited zero after guest poweroff.

First-boot journal markers:

```text
FUNGOS_MYCELIUM_PACKAGE_INERT_INSTALL_VERIFIED
FUNGOS_MYCELIUM_NATIVE_UPDATE_PREWRITE_REJECTION_VERIFIED
FUNGOS_MYCELIUM_FIRST_CONTACT_SANDBOX_VERIFIED
FUNGOS_MYCELIUM_PACKAGE_REPAIR_VERIFIED
FUNGOS_MYCELIUM_PREINSTALLED_CLAIM_AWAITING_REBOOT
```

The claim and isolated local authority were created privately **after** boot;
the actual package adapter ran under the real first-contact service sandbox.
Ownership and genuine executable digest were checked before enrollment.
Second boot emitted `FUNGOS_MYCELIUM_PACKAGE_REBOOT_VERIFIED`, confirming stable
peer/Iroh/certificate/configuration/SSH-host identity hashes and daemon health.
First contact skipped because its consumed runtime claim was absent.
Installed Debian binary/source inventory: 215 rows, SHA-256
`b77643e472264cb5046ca946772ec46e1de907cc34ca588155c10760ed3cead3`.

## Evidence and boundaries

Retained Agora cache: `/home/ajmwagar/.cache/fungos-mycelium-finaliso.2SOtCI`.
The ISO, manifest, `uefi-proof/` evidence and private `claim-proof/` clone/journals
remain there. **Do not publish the private clone.**

| Log | SHA-256 |
| --- | --- |
| `uefi-proof/install.log` | `a149d5cbd7ff61007c916e1b02cfc37e8f0fb29cadb4aa6d32fb3e4fc246bca3` |
| `uefi-proof/boot.log` | `5735fbde6966b699bde5bc20ed3a661c12b42c28fc4d0c2bdf462c056d54986b` |
| `claim-proof/boot1.log` | `1ee99da165e118d69887fe97778bf19720129c8c046076773624cbd8502a1d93` |
| `claim-proof/boot2.log` | `fed416504918b968d36197d101ab980fa02ecf4cf719a09c008e668c2eb6870b` |

Earlier `e422796` ISO proof was preparatory only; installed-purpose retention
was not in those bytes. BIOS and fully interactive installation of these final
bytes remain unverified. The known Debian AF_VSOCK generator warning remains.
No Unibus, physical media, production identity, fleet upgrade, publication or
compliance clearance is claimed. Personal SSH and signed APT lifecycle are
separate qualifications.
