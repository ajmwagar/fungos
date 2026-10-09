# Initial source import

The base builder and package/profile declarations were imported from
`ajmwagar/mycelium`, commit `fbb768b`, paths:

- `fungOS/base/`
- `fungOS/tooling/shroudoci.capabilities`

These are first-party distribution integration sources. They are now published
under `MIT OR Apache-2.0` by the project owner. The original Mycelium working
tree has not been deleted or rewritten; downstream path migration remains
separate work.

The public website stays in `ajmwagar/fungos-web`. Screenshots were copied from
that website's existing public assets, with provenance retained here.

This is a curated source import, not a dump of the development checkout.
It excludes live claims, peer identities, signing keys, credentials, generated
images, boot logs, machine-specific service units and private infrastructure
inventories. QEMU, Pi, signed-package and cloud qualification tooling will be
migrated in reviewed slices after their path and input boundaries are checked.

The `packages/` Rust publisher was subsequently imported from
`mycelium/fungOS/packages/` and extended with compressed/by-hash indexes,
explicit signed expiry, generation inventories and isolated Debian client tests.
Only source and the Cargo lockfile were imported, not generated repositories,
signing material or host-specific qualification artifacts.

The `installer/` wrapper was imported from Mycelium's isolated
`feat/fungos-installer` work, commits `f3c1a0e` and `93f4630`;
`release/` comes from `feat/fungos-release-torrents`, commit `9eac4b5`.
Only scripts, Rust source, Cargo metadata and runbooks were migrated. Retained
QEMU evidence and generated images remain outside this repository. Historical
BIOS proof in the installer runbook belongs to its recorded source and digest,
not to a new build using this checkout's shared baseline. Neither migration
imports a private identity, source closure or release approval.
