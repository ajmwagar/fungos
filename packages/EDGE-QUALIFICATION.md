# Mycelium edge APT qualification — 8 October 2026

A separate amd64 QEMU/KVM guest passed the production Mycelium CLI's explicit
APT plan/apply path for native Canvas and Unibus. Two vCPUs, 2 GiB RAM, no NIC,
no imported fleet identity, no modification of the streamed edge VM. The
successful guest rebooted from its persistent disk and powered itself off.

Verified serial markers:

```text
FUNGOS_EDGE_APT_UPGRADE_VERIFIED unibus-core 0.1.0+aptqa2
FUNGOS_EDGE_APT_FAILED_UPDATE_ROLLBACK_VERIFIED unibus-core
FUNGOS_EDGE_APT_UPGRADE_VERIFIED canvas-linux 0.1.0+aptqa2
FUNGOS_EDGE_APT_FAILED_UPDATE_ROLLBACK_VERIFIED canvas-linux
FUNGOS_EDGE_APT_CONFFILE_VERIFIED
FUNGOS_EDGE_APT_REBOOT_VERIFIED
```

The harness rejects a substituted plan digest before each apply. Good versions
share the supplied owner-built binaries; unhealthy versions contain an explicit
exit-1 executable. These are package lifecycle/recovery fixtures, not different
production application releases or an application-data migration test.

Unibus readiness performs authenticated schema-2 registration through its local
health socket. Canvas runs its native compositor without a DRM output and answers
the bounded owner-local JSON control probe. Both probes and executing binary
digests are checked through Mycelium's existing service adapter, including after
reboot. This is not a visual desktop, media-delivery or physical Pi qualification.

The initial Canvas baseline exposed a missing cursor theme. Its owner package
now declares `adwaita-icon-theme`; the successful guest explicitly included it.
Library dependencies are derived by Debian, not guessed from architecture. The
final owner packager built an archive with that dependency and passed two tests.
Mycelium's APT module passed four unit tests and Linux CLI/daemon type checks.
The source export used the existing isolated update-safety checkout and its
resolved lockfile; unrelated edits in the live Mycelium checkout were excluded.

Evidence retained on the Linux qualification host:

- Console SHA-256: `bae7255a6c790180e844a02101818adea307e6c088fbf5599d71486d30cf7ef7`.
- Signed generation ID: `b23e5052ffe79cc7cf0fc4364c04c413cacdac8452ed53dab9f1ac4f5131466a`.
- Successful disk/log: `~/.cache/mycelium-apt-qemu/guest-v4/`.

No production signer, cloud deployment, fleet rollout or automatic APT update
policy was enabled. Interrupted APT recovery remains explicit, not automatic
daemon startup recovery. Multi-package changes and maintainer-script/data
migrations remain outside the qualification contract.

## Reproduction

Use the prior disposable APT disk described by Mycelium's
`fungOS/packages/README.md`, never a managed-node disk. Build the Mycelium CLI,
owner packages and these three Rust binaries on Linux. `edge-apt-fixtures`
accepts an owner archive, package name, owner executable and fresh output path;
generate both package fixture sets and publish their six `.deb`s using the
canonical signed-generation publisher and a disposable test signer.

`edge-apt-image` takes the disposable base disk, fresh output directory,
Mycelium binary, verified repository tree, fixture directory and `edge-apt-proof`
binary. Run it with root and a writable `TMPDIR`; it clones the input, mounts
only its new artifact and suppresses service startup during dependency assembly.
The build host had broken DNS; the explicit `FUNGOS_BUILD_DEBIAN_IP` override
was used only after verifying the Debian endpoint over HTTPS. Do not persist an
unverified address or turn this workaround into a production DNS configuration.

After assembly, grant the unprivileged QEMU runner ownership of only its new
directory and disk. Boot with the existing Genesis kernel/qualification initrd,
`fungos.root=LABEL=FUNGOS_ROOT`, DMI product `fungos-edge-apt-validation`,
`-nic none`, serial logging and no shared host filesystems/sockets. Retain all
markers and clean poweroff as success evidence; a failed unit is not success.
