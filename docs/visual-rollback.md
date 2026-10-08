# Visual edge rollback qualification

This is a destructive test of a **disposable offline copy**, never an installer
or a command to run on a fleet node. It uses the production Mycelium native
activation path for real Canvas and Unibus, with signed exit-42 candidates.
It does not replace those applications with simulated health servers.

## Preparation

1. Select a stopped persistent edge image. Do not copy or mount a running guest's
   writable filesystem. Keep the original guest and TV receiver untouched.
2. Create a fresh sparse copy under an owner-only qualification directory.
   Record the kernel, initramfs and source image. Mount only the new copy.
3. Build `packages/src/bin/visual-rollback-proof.rs` and the deliberately failing
   executable for the guest architecture on Linux. The proof binary uses only
   Rust's standard library; a static musl build avoids host-glibc incompatibility.
4. Install the proof and failing executable under `/opt/fungos-proof`, the shared
   brand PNG as `undergrowth.png`, and a compatible Canvas owner CLI as
   `/usr/local/bin/canvasctl`.
5. Generate a disposable release signer using `mycelium releases keygen`. Install
   its private key as `/opt/fungos-proof/signer`, mode 0600. Configure only this
   copy's Mycelium service to trust its public key through an additional local
   `EnvironmentFile`. Retain the prior trusted public release key for rollback.
   Never copy a production signer into a guest or publish test manifests from a
   fleet daemon. Remove disposable private signer copies after the test.
6. Install and enable
   [the proof unit](../packages/fungos-visual-rollback-proof.service). Disable this
   copy's automatic-update timer to prevent competing activations. Unmount it.
7. Boot with QEMU/KVM, the existing kernel/initramfs, persistent root argument
   `fungos.root=LABEL=FUNGOS_ROOT`, **`-nic none`**, no shared filesystem or host
   sockets, and SMBIOS product `fungos-visual-rollback-only`. The Rust executable
   rejects rootless execution and any other product marker.

The qualified display uses two vCPUs, 2 GiB RAM and Bochs DRM. Expose VNC only
on loopback for read-only framebuffer evidence. For this no-NIC fixture only,
`systemd.mask=systemd-networkd-wait-online.service` avoids waiting for a network
that cannot exist. This is not a production boot-policy change.

## Assertions

The proof waits for fresh reconciliation, active native services and the Canvas
owner's successful inspect—not service activity alone. It imports and saves the
wallpaper, then records workspace, certificate and machine-ID digests.

For each of `canvas` and `unibus-router`, it signs and publishes the failing
candidate into `visual-rollback-only`, activates it explicitly, requires the
rollback error, and compares the recovered release link and executable bytes
with the baseline. The restored native service must be active and its pending
activation checkpoint cleared. Production activation separately performs the
PID-bound application readiness verification.

After both failures it requires application readiness, unchanged saved state,
and a guest reboot. On the next boot it compares retained state again and leaves
a bounded capture interval before clean poweroff.

Success requires all four journal or serial markers, a genuine framebuffer capture after
reboot, and clean poweroff:

```text
FUNGOS_VISUAL_ROLLBACK_VERIFIED canvas
FUNGOS_VISUAL_ROLLBACK_VERIFIED unibus-router
FUNGOS_VISUAL_ROLLBACK_STATE_RETAINED
FUNGOS_VISUAL_REBOOT_RETAINED
```

Any failed unit, missing marker or failed recovery is a failed qualification.
Retain failed logs too. Do not delete a pending checkpoint to make a test pass.

## Boundaries

This tests single-package native activation and previously enrolled state in an
offline clone. It is not a fresh claim/PXE test, automatic APT rollout, whole-set
atomic transaction, application-data migration test, Pi qualification, or proof
of TV media delivery. Do not run APT and native activation as competing owners
of the same executable. Production compositor updates remain manually gated on
an application-owned renderer/output health check.

## Qualification: 8 October 2026

Both signed failing native candidates passed verified rollback in the visual
clone. The saved wallpaper, machine ID and enrolled certificate survived both
failures and the following reboot. The guest then powered off cleanly. A further
cold boot repeated retained-state verification and produced a genuine 1280×800
framebuffer showing the restored wallpaper before another clean poweroff.

The initial run exposed the offline network wait. Its interruption happened
after Canvas activation had begun; the next boot reported `restored interrupted
packages: canvas`. The harness then exposed an active-service/owner-socket race,
fixed by waiting for actual owner readiness. Failed logs are retained, not
discarded. The passing normal rollback run is separate evidence from that
interrupted recovery.

Evidence on Agora: `~/.cache/fungos-visual-rollback.cqub5Q/` (private; copied peer
state and signed test candidates mean this disk must never join the mesh or be
distributed). Disposable signer copies were moved out of the guest and into
host trash after qualification; no production signer was used. Both original
QEMU guests and the TV were unchanged by fault injection.

- Proof journal SHA-256:
  `34fd799874c0c5241d6aad9bfd99e0b15378473d16b5e6e65a1481880339225e`.
- Passing rollback/reboot serial log:
  `e6ae7ff11a737670436a01316b2959c9c3f43a03649feec839b9ea17ae84adca`.
- Final cold-boot serial log:
  `a18408324fa2748cedb121f473106c72970963f26c35d6d824d56d67f9f69f35`.
- Linux musl proof executable:
  `36b903c5f2c95f21544496e7d088a10f815345a85ccc9843ca304cb6b9494e33`.
- Locally retained recovered-desktop PNG:
  `26d087bd0cfe3853fba92e90f16964eda13a927e6a0b44cedb86ec1c49ed0eb6`.

The first passing run logged markers through journald; the fixture unit now
targets the serial TTY explicitly. Captures remain unpublished pending review.
