# fungOS base image

This directory builds the reference fungOS host filesystem: a small Debian
`minbase` with systemd-networkd, explicit udev/kmod hardware discovery, SSH,
trust roots, and deliberately narrow
integration points for enrollment and signed updates. It supports `amd64` and
`arm64`; application, GPU, and board-specific packages belong in other
profiles.

The build is pinned to a Debian snapshot. Package inputs are derived from the
capabilities named by `profiles/base.capabilities`, so package ownership stays
with the capability that requires it.

## Quick start

```sh
cd base
./scripts/check.sh
sudo ./scripts/build.sh amd64
sudo ./scripts/inspect.sh out/fungos-base-amd64.tar
```

See [RUNBOOK.md](RUNBOOK.md) for prerequisites, cross-architecture builds, and
manual inspection. See [interfaces.md](interfaces.md) for the stable runtime
contracts.
