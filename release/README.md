# fungOS release preparation and torrent distribution

Release preparation stages boot-tested image bytes and corresponding source,
then asks upstream Transmission to create public, single-file torrents. It does
not publish files, start a seed service, sign metadata, or enable automatic
image installation. Mycelium remains the owner of identity and signed software
updates; BitTorrent is another way to obtain matching bytes, never authority.

The [notice-bearing Mycelium-only qa6 report](NOTICE-PACKAGE-QUALIFICATION.md)
records the actual package/ISO, fresh BIOS/UEFI tests and verified complete source
archive. Engineering coverage is reviewed; signatures and public delivery remain
separate. It is not a full edge release or legal/compliance certification.

## Release identity and package selection

Use immutable names containing the release, profile, architecture and package
variant, for example `fungos-2026.10-experimental-cloud-amd64-standard.img.xz`.
Do not overwrite an artifact or reuse a release URL for different bytes.
`standard` is intended to exclude AGPL packages; `with-agpl-apps` explicitly
includes them. These are package-selection variants, not new licenses or OS
forks. Debian's GPL, LGPL and other licenses still apply in both. These variants
are planned until the build pipeline checks every included package's license.
Unknown or custom licenses require review; absence of the string `AGPL` alone
does not prove an artifact satisfies the standard selection.

## Inputs and preparation

Install the upstream `transmission-create` and `transmission-show` tools on the
release builder. Linux also needs `sha256sum`; macOS uses `shasum`. The Rust
utility has no third-party crate dependencies and is an independent workspace.

```sh
cargo check --manifest-path release/Cargo.toml
cargo test --manifest-path release/Cargo.toml
cargo test --manifest-path release/Cargo.toml -- --ignored
cargo build --release --manifest-path release/Cargo.toml
```

The ignored tests require Transmission 4.1+ creator/show/daemon/remote tools.
They create synthetic fixtures, verify image/source torrent preparation, and
start an isolated loopback-only daemon to check full local verification and
active torrent state across graceful restart. DHT, peer exchange, LAN discovery,
WAN address probes and port mapping are disabled. Processes and fixture files
are cleaned up. This is not a peer-download or public reachability test.

Prepare six nonempty regular files before running the tool:

1. The exact image, with SHA-256 retained from its boot test.
2. An archive of complete corresponding source for the shipped components,
   including patches and required build/install scripts. A GitHub link or
   first-party checkout alone is not a Debian source closure.
3. License notices for the included components.
4. Installed package and binary inventory: names, versions, architecture,
   source package/revision, license and artifact digest where applicable.
5. Build provenance: pinned package snapshot, source revisions, build commands,
   bootloader/kernel and relevant tool versions.
6. Boot evidence tied to this image digest: target, kernel, reboot results and
   workload test outcomes. Unsupported tests remain unsupported, not passed.

The following arguments are illustrative local paths; replace them with the
actual reviewed files and tested digest. Omit the final URL until an immutable
HTTPS download origin is selected. Providing it records a web seed in the
torrent; the utility does not upload or verify that origin.

```sh
release/target/release/fungos-release \
  /srv/candidates/fungos-cloud-amd64.img.xz \
  "$TESTED_IMAGE_SHA256" \
  /srv/candidates/fungos-sources.tar.xz \
  /srv/candidates/LICENSES.txt \
  /srv/candidates/packages.tsv \
  /srv/candidates/build.txt \
  /srv/candidates/boot.txt \
  /srv/releases/fungos-experimental-cloud-amd64 \
  https://downloads.fungos.dev/experimental/cloud-amd64/
```

Preparation refuses an existing output directory, hidden/unsafe filenames,
symlinks, empty inputs, checksum mismatches and filename collisions. Keep
inputs quiescent throughout preparation. It copies explicit files only, never
an arbitrary directory. Image and source each get a `.torrent`; `SHA256SUMS`
covers both torrents and all six inputs. Transmission's anonymize option omits
creation metadata. Torrents have 1 MiB pieces and no private flag or required
tracker; optional HTTPS web seeds provide a download fallback.

`PREPARING` identifies partial output after failure. Retain it for diagnosis;
do not seed or publish that directory. `PREPARED` means local preparation
completed, not release approval. The utility checks file shape and bytes, not
whether the source/license/evidence files are substantively complete.

## Approval and publication

Before publication, inspect the mounted image for credentials, claims, private
keys/certificates, machine identity, saved tokens, shell history and private
logs. Generic public trust anchors and first-boot enrollment adapters are
different from embedded credentials. Remove test SSH authorization and
machine-specific identity; test the final sanitized bytes again and record
their digest. Check license inventory and corresponding-source completeness.
Scan evidence accompanies the release; a successful boot is not vulnerability
clearance or compliance certification.

Sign the release digest/inventory through the existing authorized release
authority, not a new torrent-specific key. Whole-image signature publication
and installer verification are not implemented by this preparation utility;
do not expose an unsigned image as an authenticated production release or feed
it into automatic installation. Publish source, licenses, inventory, evidence,
checksums, signature and image together at an immutable HTTPS location. The
download page must state experimental status, target compatibility, signature
verification and known limitations. Do not list unavailable files as downloads.

Verify bytes fetched from the real HTTPS location against the authenticated
digest before activating its web seed. Do not use expiring authenticated URLs
in public torrent metadata. SHA-256, torrent piece hashes and magnet links
identify bytes; they do not establish who approved them.

## Seeding and independent verification

Use a dedicated Transmission daemon with its own state directory and a
restricted service user. Keep RPC bound to loopback or a protected management
socket/network, not a public unauthenticated listener. Disable automatic
UPnP/NAT-PMP mapping; expose only an explicitly selected peer port. Limit upload
rate and peer count so seeding cannot saturate inference, audio or management
traffic. Keep public seed payloads separate from private release work files.
Seeders' public IP addresses are visible to other participants; seeding is
opt-in, not a default installed-OS behavior.

After staging the exact release files on a selected seed host, add each torrent
paused with `transmission-remote`, select its exact payload directory, force
verification, confirm 100 percent local data, then start it. Record the torrent
info hash and payload SHA-256. Restart that daemon and confirm the seed resumes
with the same payload. Commands depend on the deployed Transmission version;
use its `--help` rather than guessing RPC flags.

Test with a second, empty client: fetch the torrent from the release origin,
download from the seed, verify the authenticated SHA-256 and boot the downloaded
image. Test peer-only and HTTPS-web-seed paths separately; a working web seed
does not prove P2P reachability. DHT without an initial reachable seed is not a
durability guarantee. Keep an HTTPS origin plus at least one persistent seed;
add another independent seed when a release becomes supported.

The temporary DigitalOcean boot-test droplet is not a persistent seed host and
must be torn down. Persistent hosting location, bandwidth budget and signed
image publication remain explicit rollout decisions.

Upstream interfaces: [Transmission torrent creation](https://github.com/transmission/transmission/blob/main/utils/create.cc),
[daemon configuration](https://github.com/transmission/transmission/blob/main/docs/Editing-Configuration-Files.md).
