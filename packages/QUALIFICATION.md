# APT qualification — 8 October 2026

Linux `cargo check --locked --all-targets` and `cargo test --locked` passed:
two unit tests and one end-to-end integration test.

The integration test generates a disposable signing key and three control-only
packages: two amd64 versions and one arm64 version. It verifies the generation
manifest, retained versions, by-hash copies and signed expiry. An isolated real
`apt-get update` accepts the admitted key and rejects a nonmatching key and
tampered indexes. No packages are installed and no host APT configuration is changed.

The corresponding `shared-infra/projects/fungos/apt` root passes OpenTofu
validation and four mocked, plan-only tests. It is registered with
`applyEligible=false`: the hosted storage API and object-delivery binding still
require platform qualification. The wider shared-infra inventory check fails on
existing nested worktrees and unrelated unregistered roots.

No cloud origin, production signer, DNS change, mutable update channel or fleet
rollout is established by these tests. The initial handoff pins a generation;
automatic channel promotion and expiry renewal remain deployment-owner work.
