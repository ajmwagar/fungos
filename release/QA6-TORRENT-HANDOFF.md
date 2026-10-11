# QA6 torrent handoff

Prepared locally on 2026-10-10. Not signed, uploaded or seeded. These are the
Mycelium-only amd64 qualification artifacts, **not fungOS v0.1.0 or a full edge
image**. Fresh BIOS and UEFI installation and corresponding-source coverage are
recorded in [NOTICE-PACKAGE-QUALIFICATION.md](NOTICE-PACKAGE-QUALIFICATION.md).

The two generated torrent files are retained in `release/evidence/`. Their
SHA-256 checksums and the exact payload checksums are recorded in
`evidence/mycelium-qa6-torrents.SHA256SUMS`. This checksum file is **unsigned**;
neither its presence nor a torrent infohash proves release authority.

| Payload | Bytes | BitTorrent v1 infohash |
| --- | ---: | --- |
| fungos-mycelium-qualification6-amd64.iso | 833945600 | 020aa0817773245d1ebd9247b4d47bbfda0ed1e9 |
| fungos-mycelium-qa6-sources.tar.zst | 3271330118 | c42a6334d7b1ad840e6f5fd258164dba03d5a6fb |

Transmission 4.1.1 generated and parsed both single-file torrents. They use
1 MiB pieces (796 image pieces; 3120 source pieces), no tracker or webseed,
and no creator or creation-time metadata. The public-torrent flag permits a
future public swarm; it does not mean a swarm is currently running. Both
payload SHA-256 hashes were independently rechecked after generation and match
the qualified artifacts. A final-image two-client download has **not** been
performed; the earlier isolated synthetic transfer is separate evidence in
[TORRENT-QUALIFICATION.md](TORRENT-QUALIFICATION.md).

## Artifact location and reproduction

Payloads and matching torrent copies remain on Agora at
`/home/ajmwagar/.cache/fungos-notice-qa6.lzDJzr/` (torrents under `torrents/`).
No multi-gigabyte staging copy was made: Agora has approximately 1.1 GiB free.
The repository preparation CLI remains unchanged and defaults to copying its
inputs; do not run that copying workflow here without sufficient space.

The manual equivalent uses the same upstream creation arguments as that CLI:

```sh
transmission-create --anonymize --piecesize 1024 \
  --outfile NEW_IMAGE_TORRENT QUALIFIED_IMAGE
transmission-create --anonymize --piecesize 1024 \
  --outfile NEW_SOURCE_TORRENT COMPLETE_SOURCE_ARCHIVE
transmission-show NEW_IMAGE_TORRENT
transmission-show NEW_SOURCE_TORRENT
sha256sum QUALIFIED_IMAGE COMPLETE_SOURCE_ARCHIVE
```

Use fresh output paths, immutable/quiescent inputs and the recorded payload
digests. Generation does not launch a daemon or contact a tracker. Creating
webseed variants later changes torrent-file checksums; record those variants
separately rather than overwriting this evidence.

## Publication boundary

Before any public release, use the existing authorized release signer, select
an immutable hosting origin and explicitly authorize the seeder and its
bandwidth. Publish the source, notices, inventories and boot/build evidence
alongside the image, authenticated checksums and signature. Verify a separate
empty client's actual downloaded payload against authenticated checksums and
boot that download. Follow [README.md](README.md) for seeder isolation and
limits. Do not advertise download links before those files are available.

This preparation makes no legal, CVE, STIG or production-readiness certification.
