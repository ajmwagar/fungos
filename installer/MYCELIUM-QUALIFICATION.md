# Mycelium-only installer qualification

## Corrected candidate: ownership and enrollment gates passed

The current tested candidate is clean fungOS source
`b63a4e1fbbfb76921823f6f89eabff8ce77b19cb`, ISO SHA-256
`601a631bb41cd93ef091e9b6a38b25d09bb49395a036f0bf668053dc9bde0b5c`.
It embeds genuine `mycelium_0.1.0+qualification5_amd64.deb`, SHA-256
`db6c2e929b127fa89146175ff063fb0bf43c1785ef9a08a398aee4230b218daa`,
from source `10a3c18ebea8b9f3f7be981657957ac10a5a2f16`. The installed
executable SHA-256 is
`60664012ba7b7b3a7c3ece05975de582caa9b6cc207a4f1b3ce535f17d3c2fa8`.
Its narrow purpose and snapshot remain unchanged; it is not a full edge release.

Both guarded fresh 8 GiB BIOS and UEFI installs emitted
`FUNGOS_INSTALL_COMPLETE`, then booted natively from the installed disks alone
to `fungos login:`. Both harnesses exited zero. Read-only inspection of both
stopped disks confirmed `/`, `/etc` and `/etc/ssh` are root-owned `0755`,
Mycelium is initially unenrolled, and no runtime claim environment is present.
All remastered media file checksums passed, including the genuine package and
shared overlay copier; no claim/private-key payload was added.

This fixes a real installer defect, not just a test fixture: the preceding
`b332335d971d49c7b8cc0670073c8426eadd8d6b6c9a124474e687d20be7078c`
ISO installed those ancestors as UID/GID 1000 and mode `0775`. The former
`cp -a` overlay application retained builder checkout metadata. The shared
copier now normalizes only staged public overlay nodes. Its root regression
verified that unrelated target private files retain their owner, mode and
contents. Earlier installer results below are historical, not release approval.

A private clone then ran genuine first contact under the real service sandbox,
native-update pre-write rejection, and package repair. Its first native UEFI
boot emitted `FUNGOS_MYCELIUM_PREINSTALLED_CLAIM_AWAITING_REBOOT`; the second
boot of the same disk and firmware variables emitted
`FUNGOS_MYCELIUM_PACKAGE_REBOOT_VERIFIED`. Peer, Iroh, certificate, configuration
and SSH-host identity hashes stayed stable; the daemon passed a real RPC health
check and first contact skipped the consumed claim. Both boots powered off
successfully. The proof derives the expected executable from the pinned actual
package payload, not the old historical binary pin (`149d64b` / `c20022f`).

Retained evidence: Agora
`/home/ajmwagar/.cache/fungos-final-overlay-qa5.ngobmG`.
The `claim-proof/` disk contains isolated QA identity and must never be published.
All owned QEMU processes stopped and inspection loops were detached before
handoff to the separate personal SSH and signed-APT qualification work.

| Log | SHA-256 |
| --- | --- |
| `uefi-proof/install.log` | `6a4b3f94bb481e13a95d0fa4a6c54ab61f8165cca41893887b19c3ac59e07b92` |
| `uefi-proof/boot.log` | `3d119372f995fcee86928ebc10aa367f739ba7d6dc20411a567814a55634e4c7` |
| `bios-proof/install.log` | `cb22653160062b3493b50312549e42b919131b8419fa3f35e9dc35fc57f5db43` |
| `bios-proof/boot.log` | `59223644e4bbc2986da8ea6f32e4796cc30c7dbbed5d3813b0aac6c5f5011925` |
| `claim-proof/boot1.log` | `ef88b69803bbbb3a975b70d63c65e5e2f66316e7cf4b432c9d87fcbb87c34169` |
| `claim-proof/boot2.log` | `ce5742e3d7e893a7b42cc6d7e15e8d5e0109f6764434a716d3e659f496175aab` |

Public `release-inputs/installed.tsv`: 215 rows, SHA-256
`d055de76fe467b863efa66fb8b3ce75a8c2ffb17e687a3a096ea4ad7fba7db5a`.
BIOS inventory: 207 rows, SHA-256
`d67b077a9dce76b910954087205ee3901afd4e3e9b3b4189f37aac13a6a437be`.
Copyright archives contain only package notice paths, not private state. These
remain source-closure inputs, not a claim of complete corresponding source.
Personal SSH, signed-APT lifecycle, full interactive installation and release
publication are separate gates; this evidence does not claim them passed.

## Historical source138 candidate

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

The same exact ISO also passed guarded fresh BIOS installation and native
disk-only boot with kernel `6.12.111+deb13-amd64`; its harness exited zero.
Read-only inspection confirmed Mycelium enabled but unenrolled, no claim
environment, and retained narrow purpose. BIOS inventory: 207 rows, SHA-256
`6787dadeed12cc380c291f4561cb8dbe0c95c388b60a05cf953b9cc21f0b910c`.
BIOS log hashes: install
`c1763aafc9a98e6c6b196e472cd859d12e2df90a9550acf54bc64463bd3c6b4c`,
boot `796acc5722594c6c1bcf73b1d0fc16e3e72e855a0b05ac2aea2a7bba83c2a82f`.
Public inventories and package copyright notices were exported separately to
`release-inputs/`, never from a private home or identity directory. These are
closure inputs, not complete corresponding source: the netinst media includes
installer/kernel packages beyond the installed inventories.

Earlier `e422796` ISO proof was preparatory only; installed-purpose retention
was not in those bytes. Fully interactive installation remains unverified.
The known Debian AF_VSOCK generator warning remains. A later SSH-fixed package
must be included in a new ISO and qualified again; these results are not
transferable to substituted package bytes.
No Unibus, physical media, production identity, fleet upgrade, publication or
compliance clearance is claimed. Personal SSH and signed APT lifecycle are
separate qualifications.
