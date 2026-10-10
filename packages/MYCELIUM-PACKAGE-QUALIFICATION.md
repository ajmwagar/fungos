# Mycelium owner-package qualification

Status: genuine owner packages passed the isolated install/enrollment/repair,
Debian lifecycle and reboot gates below on 2026-10-09. Personal SSH access and
the signed-repository update engine remain unverified; this is not a complete
operational release image qualification. Unibus is excluded pending its
independent licensing
decision. Baseline installer evidence is in
[../installer/QUALIFICATION.md](../installer/QUALIFICATION.md).

Run on a fresh disposable Linux/QEMU disk, not Agora's host rootfs or any fleet
node. Use an isolated test authority and runtime-delivered claim. Never put a
private identity, authority key or claim in public media. Record the exact source
commit, Debian package/version/architecture/SHA-256 and complete test logs.

## Acceptance gates

1. **Inert install:** install the genuine owner `.deb` with APT/dpkg. Verify
   `/usr/bin/mycelium` and the native systemd unit are owned by that package,
   match the archive payload, and pass owner self-check. An unenrolled image has
   no node/private identity, no running daemon and no generated SSH trust.
   Package installation must not create an authority or silently enroll a peer.
2. **Package-safe claim:** deliver a short-lived test claim only after boot and
   enroll through the public CLI's package mode. Verify the daemon runs the
   package-owned executable and unit, with writable enrollment state under
   `/var/lib/mycelium`. Before/after SHA-256 values of the executable and native
   unit must match. Neither `/var/lib/mycelium/bin/mycelium`, a per-user managed
   binary, `/etc/systemd/system/mycelium.service` nor a generated shadow unit may
   appear. No user-systemd service should be created by package enrollment.
3. **Repair convergence:** repeat setup/service repair in package mode. Preserve
   peer signing, Iroh endpoint and certificate identity; preserve owned-file
   digests. Reject an incompatible state path or missing native package/unit
   before copying a binary or writing identity/configuration.
4. **Ownership enforcement:** attempt native self-update against the
   package-owned executable. It must reject before changing the executable,
   unit, package state or native activation/backup state. Exercise the real
   explicit-write path, including a destination symlink/alias. The package
   ownership guard runs before release retrieval, so its rejection can be
   qualified without publishing a signed release; this is not release-signature
   validation. The underlying activation entry point must enforce
   the same guard before any backup/failed-health rollback path can run. There
   is no public native rollback CLI command; rejection of an unknown command
   proves nothing about ownership. Do not authorize native automatic activation
   for this APT-owned component.
5. **Real Debian lifecycle:** install a second genuine owner package version,
   then explicitly restart the native service. Check Debian's installed version,
   actual running executable digest and health. Downgrade to the retained first
   genuine package and restart again. Identity/configuration survives both; no
   native copied executable or unit shadow appears. Two metadata versions with
   the same executable can qualify package lifecycle, but must be disclosed and
   cannot prove application compatibility or data-migration rollback.
6. **Reboot persistence:** boot from the installed disk alone, verify the native
   service, peer identity, Iroh endpoint identity, certificate identity and SSH
   host key persist. Package ownership and executable/unit hashes remain correct.
7. **Access separation:** verify personal certificate-only SSH using an active
   isolated grant and reject an ungranted person. Peer enrollment alone must not
   create a personal SSH grant or grant sudo. An accepted local package or peer
   health check does not prove access policy.

## Evidence and recovery

Record applied, healthy and recovered as separate outcomes. Keep the prior `.deb`,
its digest, disposable disk, native service journal, enrollment result and owned
file/identity hashes. Exclude claim codes, tokens and private key contents from
logs. Stop owned QEMU processes and detach read-only inspection loops afterward.
If enrollment or an upgrade fails, preserve the failed test disk; retry only on
a new disposable clone. Never repair a failed test by replacing the package
binary with a native copied binary or bypassing certificate verification.

This gate covers Mycelium package ownership and enrollment, not Secure Boot,
TPM protection, public APT hosting, Pi hardware, or Unibus redistribution.

The private Rust helpers `src/bin/mycelium-package-image.rs` and
`src/bin/mycelium-package-proof.rs` assemble a clone of an unenrolled installer
qualification disk and run these checks inside it. Inputs are genuine owner
packages, never fixture executables. The assembler refuses block devices and
symlink inputs. The proof refuses a guest without its exact qualification DMI
marker, generates an isolated runtime authority, and delivers the claim only to
a private `/run` file. These helpers do not qualify the signed-repository
Mycelium APT `plan`/`apply`/`recover` engine or personal SSH grants.

## Verified private guest run: 2026-10-09

Package executable, assets and packager came from the same clean Mycelium source
`138feeb298174b9158e03bf2c76692572ce3e7df`. Both are genuine static-musl amd64
owner builds, not fixture binaries:

| Artifact | SHA-256 |
| --- | --- |
| `mycelium_0.1.0_amd64.deb` | `67db12ef7ea04b34e9a7ea9125d5bd563ee22f378244db00f41b82cf1c67a556` |
| `mycelium_0.1.0+qualification2_amd64.deb` | `491a0c590030ee34ecc47fd53f833489ef8a56b042701c6c7150637d5e2c0cc7` |
| Package executable | `6956b35956332a897ba7f398e019113b91b6f211bb0ce657e266ab54b11733fa` |
| Native systemd unit | `cd10657d4feac46fcdbce594f3ce0dade7ca2fdff8b2adec1a24dbd05fd02051` |

A and B intentionally contain the same executable. The test proves real Debian
version transitions, retained private configuration and explicit native-service
restart, not changed-code compatibility or application data-migration rollback.

The private proof source was fungOS
`917de0d80a61f9879ef05d6a65b9052e8d3166b0`; the final clone assembler source
was `505d0cdf2d177d5cb783a61eae1c2f05bb889bca`. On Agora, artifacts are under
`/home/ajmwagar/.cache/mycelium-deb-source138/artifacts` and private qualification
evidence is under `/home/ajmwagar/.cache/fungos-package-proof.yiSXTq`.
`guest-v5/disk.raw` is a clone of the refreshed Trixie installer qualification
disk, not a public release image. No production identity or fleet node changed.
The guest had no NIC/backend, host forwarding or shared filesystem. A QA-only
dummy interface with `192.0.2.1/32`, without routes, satisfied its normal
network-online dependency. Enrollment used a fresh in-guest authority and a
short-lived loopback claim stored only in private `/run` state.

The retained first-boot journal records all seven acceptance markers through
`FUNGOS_MYCELIUM_PACKAGE_AWAITING_REBOOT`: inert install, guarded native explicit
write (including an alias), the actual first-contact service with
`ProtectSystem=strict` and `ProtectHome=yes`, repair, genuine APT upgrade and
downgrade, with package-owned binary/unit and unchanged private identity hashes.
The same disk's second native UEFI boot records
`FUNGOS_MYCELIUM_PACKAGE_REBOOT_VERIFIED`. Its first-contact unit correctly
skipped because the successfully consumed runtime claim was absent; the native
daemon was healthy and used the package executable.

| Retained evidence | SHA-256 |
| --- | --- |
| `qualification-v5-boot1.journal` | `341f093a137e8c40ccf215246c88b8323fc7dbc4c8992da38f3957d8e122f217` |
| `qualification-v5-boot2.journal` | `b1d97cad4d69e1a237a365af5b314d0de9fc3c8203515bb5946d546079db7b50` |
| `guest-v5/boot1.log` | `b494a559f2464728f6419bd70a10deac12c0f05f079f82ab1192a26c548f9934` |
| `guest-v5/boot2.log` | `bbf4e0bd4f2282e0c78a51bf02a9f14308bd5fe84f2ddad97b3b2f87d728caef` |

The snapshot of peer signing key, Iroh key, node certificate/key, node config and
SSH host public-key hashes remained identical through repair, A→B→A and reboot;
its private receipt SHA-256 is
`e18e38d033a3b4ca33344eec4a84c22eb5e9d97538f08de59b5c59c385f43925`.
Key contents and claim tokens are not published. Both successful boots powered
off; all owned qualification QEMUs stopped and inspection loops detached.

Earlier private attempts are preserved but not accepted: no-NIC online gating
prevented execution, `apt-get --no-download` rejected local staging, and serial
getty reset the proof's stdout. The final run uses normal APT local-file install
with no external connectivity and writes its evidence to the retained journal.
The Debian AF_VSOCK SSH generator warning remains a baseline finding and is not
silently treated as a clean whole-system health result.
