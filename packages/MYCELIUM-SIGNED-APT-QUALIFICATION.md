# Mycelium signed APT qualification

Verified 2026-10-10 in a disposable amd64 fungOS guest. This is evidence for
the existing `mycelium software apt plan/apply/recover` engine, not a fleet
rollout or publication of a production APT repository.

## Final candidate requalification

The complete engine proof was repeated on the freshly installed, root-owned
UEFI disk from corrected ISO SHA-256
`601a631bb41cd93ef091e9b6a38b25d09bb49395a036f0bf668053dc9bde0b5c`,
after genuine isolated first-contact enrollment and reboot. The earlier retained
fixture had incorrect OS ancestor ownership; it is not whole-image evidence.

The final candidate was owner source
`10a3c18ebea8b9f3f7be981657957ac10a5a2f16`, package
`0.1.0+qualification5`, archive SHA-256
`db6c2e929b127fa89146175ff063fb0bf43c1785ef9a08a398aee4230b218daa`.
Signed repository generation was
`88abf08921df1043490c148f041602bcebb84673db58a4a7470398965cd0187c`.
The real engine first downgraded the installed final candidate to the genuine
baseline above (`FUNGOS_MYCELIUM_SIGNED_APT_BASELINE_DOWNGRADE_VERIFIED`), then
passed all eight markers below with the final candidate substituted for signedqa2.
Health fault injection still used signedqa3, retaining the real previous qa5
archive for rollback and recovery. All identity digests remained unchanged.

After actual reboot, the installed executable SHA-256 was
`60664012ba7b7b3a7c3ece05975de582caa9b6cc207a4f1b3ce535f17d3c2fa8`.
The bounded, no-NIC QEMU exited successfully; its read-only journal inspection
confirmed every marker. Owned loop devices were detached afterward. Helpers
include `86daaf2`, `c71c0ae` (reject wrong-owned OS fixtures), and `d215ddb`
(signed-engine baseline priming); no package or runtime implementation was
replaced by the harness. This qualifies exact final bytes, not a fleet rollout.

## Exact inputs

The prerequisite was the private, already-qualified Mycelium package disk from
[MYCELIUM-PACKAGE-QUALIFICATION.md](MYCELIUM-PACKAGE-QUALIFICATION.md). Only that
test disk was cloned; production peer identities and signing keys were not read.

| Package version | Owner source commit | Archive SHA-256 |
| --- | --- | --- |
| `0.1.0` | `138feeb298174b9158e03bf2c76692572ce3e7df` | `67db12ef7ea04b34e9a7ea9125d5bd563ee22f378244db00f41b82cf1c67a556` |
| `0.1.0+signedqa2` | `407ebc44897fe6f27225b59215485b78fd025982` | `5bc451c394a34fbcc08146fbd08e9ae6caac2cee8258075ac601a7bc45135426` |
| `0.1.0+signedqa3` | `138feeb298174b9158e03bf2c76692572ce3e7df` | `47062fab9988367ffcfbe672224b73df837d9b5b29a50b179015e552b25d8a19` |

Each package was constructed by Mycelium's owner Rust packager, from a clean
committed archive and genuine static-musl executable. The successful candidate
adds a useful `_self-check` compiled-Iroh-capability diagnostic, with a passing
source test. Its executable SHA-256 is
`a592b4626aaef20422fa7afd61f7ae4356f982307bbfab07200af3c66258826c`, different
from the baseline/recovery-test executable
`6956b35956332a897ba7f398e019113b91b6f211bb0ce657e266ab54b11733fa`.
This is changed-binary evidence, not just a Debian version-label transition.

The canonical `fungos-apt-repository` publisher generated the repository with
an isolated, one-day test signing key. Only its public key and signed generation
entered the guest. Signed-By and Valid-Until checks remained enabled; there was
no `trusted=yes`, insecure-repository flag, signature override or uplink.

- Test public fingerprint: `3F8BEF8B9ED70726C3DD3398BD6FC61279B52028`.
- Repository generation: `23424385eda3f34bea7323193822b1ffc3bf7abee3dc43ed608b97dd8cc5fd91`.
- Harness: `mycelium-apt-proof` and `mycelium-apt-image`, committed in fungOS
  `f62f268` with corrections `ec5d442` and `bcc4f49`.

## Runtime results

The guest used native UEFI disk boot, KVM/q35, 2 CPUs, 2 GiB RAM and no NIC.
The package-owned root systemd service and its service-owned Unix JSON health
socket were used without replacing the enrollment or APT engine.

All eight journal markers were observed:

1. `FUNGOS_MYCELIUM_APT_EXACT_DIGEST_REJECTION_VERIFIED`: a substituted plan
   digest was refused without changing the executable or package version.
2. `FUNGOS_MYCELIUM_APT_MISSING_SIGNED_BY_KEY_REJECTED`: apply's authenticated
   metadata refresh failed when the configured public key was removed.
3. `FUNGOS_MYCELIUM_APT_BAD_SIGNATURE_REJECTED`: altering a signed InRelease
   field made apply fail before installing the candidate.
4. `FUNGOS_MYCELIUM_APT_STALE_PLAN_REJECTED`: a locally changed health binding
   made the prior exact plan stale; restoring the authorized binding admitted it.
5. `FUNGOS_MYCELIUM_SIGNED_APT_CHANGED_BINARY_VERIFIED`: real plan/apply staged
   authenticated previous/candidate archives and installed changed candidate bytes.
6. `FUNGOS_MYCELIUM_SIGNED_APT_FAILED_HEALTH_ROLLBACK_VERIFIED`: the candidate
   failed readiness/stability, apply returned a failure, and the retained previous
   archive restored the healthy candidate from step 5.
7. `FUNGOS_MYCELIUM_SIGNED_APT_INTERRUPTED_RECOVER_VERIFIED`: interrupting a
   real apply after its checkpoint left actual recovery state; `recover --write`
   restored the previous archive and health, then removed that checkpoint.
8. `FUNGOS_MYCELIUM_SIGNED_APT_REBOOT_VERIFIED`: the same disk rebooted with the
   healthy changed candidate and no pending recovery.

The failure test is transparent QA fault injection: a bounded monitor kills only
the supervised process whose `/proc/PID/exe` bytes match the exact test candidate.
It does not fabricate a package, replace the service-owned listener, weaken a
health check, or assert the genuine binary is intrinsically broken. It proves
the recovery mechanics under observed application/process failure.

Peer key, Iroh key, certificate key/certificate, node configuration and SSH host
public key digests were unchanged across success, rollback, recovery and reboot.
The running executable stayed dpkg-owned at `/usr/bin/mycelium`; the owned unit
remained `/usr/lib/systemd/system/mycelium.service`. No home binary or /etc unit
shadow appeared. The final executable hash matched the changed candidate.

## Reproduce without a fleet

1. Build genuine owner packages from the exact source commits above. Retain
   source/license notices and verify package/executable hashes.
2. Use the canonical publisher with a fresh test-only GPG home to create one
   signed generation containing all exact versions. Do not export its private key.
3. Build the two Rust harness binaries; invoke `mycelium-apt-image` with a private
   completed package-qualification raw disk, a new output directory, the signed
   repository and the proof binary. It refuses device/symlink inputs and requires
   the earlier private package-proof receipt.
4. Boot the new disk with the exact DMI product
   `fungos-mycelium-apt-validation`, no NIC and a bounded QEMU timeout. The harness
   rejects any other host. It reboots once and powers off after verification.
5. Inspect its persistent journal read-only. Require all eight markers, the final
   changed executable digest, retained identity receipt and absent pending state.
   Detach only the loop device allocated for that inspection.

Retained private evidence on Agora:
`/home/ajmwagar/.cache/fungos-signed-apt-proof.f62/guest-v2`.
The successful first boot ran 15:57:05–15:58:00 UTC, and the reboot verification
completed at 15:58:12 UTC. The owned QEMU exited on poweroff and inspection loop
was detached. Earlier `guest-v1` is a failed harness attempt, not acceptance
evidence (it assumed compact formatting of the retained health-binding JSON).

This does not qualify a production signer, remote repository/CDN deployment,
multi-package dependency changes, maintainer scripts, ARM64, fleet rollout,
personal SSH authorization, or an intrinsic application regression. Those remain
separate gates; no production deployment is implied.
