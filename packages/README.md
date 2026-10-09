# Signed Debian repository generations

The [edge qualification](EDGE-QUALIFICATION.md) exercises Mycelium-coordinated
Canvas/Unibus package upgrades, failed-update restoration and actual disk reboot.

Debian supplies the operating system; component owners build their `.deb`
payloads. This Rust publisher calls Debian's existing index tools and GnuPG.
The [runtime component gate](COMPONENTS.md) validates owner packages before
installer inclusion and records remaining operational-release blockers.
It does not build packages, create a production signing authority, upload
objects, or replace APT/dpkg.

```sh
cargo check --locked --manifest-path packages/Cargo.toml
cargo test --locked --manifest-path packages/Cargo.toml
cargo build --locked --release --manifest-path packages/Cargo.toml
GNUPGHOME=/path/to/authorized/signing-home \
  packages/target/release/fungos-apt-repository --write \
  /new/unpublished/generation FULL_SIGNING_FINGERPRINT testing package.deb
```

Run on Linux with `dpkg-dev`, `apt-utils`, GnuPG (`gpg` and `gpgv`), `gzip`,
GNU coreutils and GNU date. Use a writable `TMPDIR` if `/tmp` is quota-limited.
The output directory must not exist. Only `amd64` and `arm64` packages are
supported in this slice; package identity comes from `dpkg-deb`, not filenames.
Duplicate destinations, symlinks, empty indexes and command failures are errors.

## Output and promotion

Each generation contains:

- `pool/`: every supplied package version, without overwriting another version.
- `dists/SUITE/main/binary-ARCH/Packages` and deterministic `Packages.gz`.
- SHA-256 `by-hash` copies of both indexes.
- Signed `InRelease`, with seven-day `Valid-Until` and hash acquisition enabled.
- `fungos-archive-keyring.gpg`: the selected signer's **public** key only.
- `SHA256SUMS`: sorted inventory of the payload and signed metadata.
- `GENERATION`: SHA-256 of that inventory; `READY` is written last, after checks.

Publish the entire verified tree under `generations/GENERATION/`. Verify the
manifest digest, every object checksum and the `InRelease` signature using an
independently admitted key before promoting an active channel. Never trust a
`READY` marker alone or accept a signing key just because the upload includes it.
Partial uploads are not release candidates. Retain previous generations and
their package/index paths; never overwrite an existing content-addressed prefix.

For the first cloud handoff, client source output pins a generation URL. It does
**not** configure a mutable `current` alias or automatic fleet adoption. Stable
channel promotion, hostname/TLS and HTTP object delivery belong to FPL Cloud.
Hash acquisition on a mutable alias also requires retaining old hash paths:
switching a prefix alone is not enough for clients holding old Release metadata.

The suite is deliberately opt-in (`NotAutomatic` plus `ButAutomaticUpgrades`).
APT remains responsible for package dependencies and conffiles. Mycelium owns
selection, rollout gates and recovery; do not manage the same executable through
both APT and native binary activation.

## Client trust

Install an independently admitted public key at
`/etc/apt/keyrings/fungos-archive.gpg`, mode 0644. Use a deb822 source with
repository-specific `Signed-By`, `Check-Valid-Until: yes` and HTTPS. Do not use
`apt-key`, `trusted=yes`, or disable signature/expiry checks. Metadata must be
reissued before seven days even if package versions do not change.

No production key or client configuration is provisioned by this tool. Bind
signing admission/rotation to the existing authority, not a second update-only
trust system. Release signing and the serving tier have separate credentials;
the serving tier must not possess the signing private key.

## FPL Cloud ownership

Deployment root: `shared-infra/projects/fungos/apt/`. It declares an existing
FPL object-storage resource and renders a generation-pinned client source. It
does not deploy a new web daemon, put signer credentials in OpenTofu state,
invent an S3 endpoint from an opaque binding, or apply infrastructure.

The original APT publisher came from Mycelium's `fungOS/packages/` qualification
work. Existing disposable install/upgrade/downgrade and reboot evidence remains
there; those older runs are not proof that this new cloud handoff is deployed.
