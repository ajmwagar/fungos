# Mycelium owner-package qualification

Status: test plan only. No owner-built Mycelium `.deb` or operational image has
passed these gates yet. Unibus is excluded pending its independent licensing
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
4. **Ownership enforcement:** attempt native self-update and native rollback
   against the package-owned executable. Both must reject before changing the
   executable, unit, package state or native activation/backup state. A dry-run
   refusal alone is not enough: exercise the real explicit-write path using a
   valid isolated signed release. Do not authorize native automatic activation
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
