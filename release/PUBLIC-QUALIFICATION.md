# Public QA6 qualification download

Published 2026-10-11 UTC at
`https://downloads.fungos.dev/qualification/qa6-a77efddc84766530/`.
The live [fungOS downloads section](https://fungos.dev/#downloads) links the
image, complete corresponding sources, both HTTPS-webseed torrents, signed
checksum inventory, public key and qualification evidence.

This is a **Mycelium-only amd64 qualification installer**, not fungOS v0.1,
the complete Unibus/Canvas edge image, a physical-hardware qualification, or
a production/security/compliance certification. BIOS and UEFI installation
evidence for these exact bytes is in `NOTICE-PACKAGE-QUALIFICATION.md`.

The published inventory is `release/evidence/webseed-qa6/SHA256SUMS`; its
detached signature and the exact torrent variants are retained alongside it.
The primary release-key fingerprint is
`FC3C04696D437DCD2F51895F0BEFDC5CF68C7CBA`. See `SIGNING.md` for custody,
verification and trust-admission boundaries. Initial signature verification
passed; an altered inventory was rejected. A separately restored protected
secret-key export also successfully signed an inventory, and its temporary
key home was removed. Off-device backup remains an operator requirement.

## Delivery evidence

- Dedicated Cloudflare R2 bucket: `fungos-downloads`.
- `downloads.fungos.dev` domain ownership and TLS both active; minimum TLS 1.2.
- Public HTTPS ISO readback SHA-256:
  `a77efddc847665307fc242bab1834522c7de3c838f486054ece9ada31a29c809`.
- Full public HTTPS source readback SHA-256:
  `79300cc84ffbe19eb547f41cdfa70764e9207b675a162c17ce385e3a39a57600`.
- Publicly downloaded signature verified with the locally admitted release
  keyring; all six smaller inventory entries passed checksum verification.
- Webseed URLs use the stable `?v=qa6` query to avoid an early cached pre-upload
  404. Verify the advertised URLs, not the preflight's negative-cache entry.
- Torrent infohashes remain `020aa0817773245d1ebd9247b4d47bbfda0ed1e9`
  (image) and `c42a6334d7b1ad840e6f5fd258164dba03d5a6fb` (sources).
- `fungos-web` main commit `55bda99` was pushed. Its live downloads section
  matches source SHA-256
  `e2ac31cadfc33110f90ee77b78da04bc8b9c1d7afca99a19acae462b64e64da0`.
  Fab injects its editor runtime into the full HTML, so whole-file equality is
  not asserted. The production CLI session returned 401; no authenticated
  build/deployment ID was obtained and no private operator bypass was used.

R2 serves files and torrent webseeds, not a BitTorrent daemon. No persistent
public peer seeder, automatic installer signature enforcement, APT channel
publication, FPL-managed storage admission or fleet trust change is claimed.
The current managed FPL adapter rejects public delivery and R2 versioning;
the operator Cloudflare path is documented in shared-infra at
`projects/fungos/downloads/README.md`. R2 has no bucket versioning or WORM
guarantee in this setup; the initial uploader is not claimed bucket-scoped.

The immutable source archive contains upstream source/test fixtures, not fleet
claims, operator keys or private guest disks. Engineering notice coverage is
documented; legal clearance is not asserted.
