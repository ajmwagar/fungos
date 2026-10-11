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

An initially empty Transmission 4.1.1 client downloaded both published torrents
entirely through their HTTPS webseeds. It was deliberately paused and shut down
mid-transfer, then restarted using its retained state. Both payloads reached
100% verified and their SHA-256 hashes matched the inventory. DHT, PEX, LPD and
port mapping were disabled; RPC was loopback-only and the peer listener used
the private LAN interface. There were zero connected peers and zero uploaded
bytes. The daemon exited cleanly after completion. This is an actual webseed
transfer and resume test, not a public peer-swarm or new native boot test.

Raw receipts are retained in `release/evidence/`:

- `mycelium-qa6-public-webseed-transfer.txt`, SHA-256
  `387126035b02f65b7c3266742e99917c420ce2965186e0ec7a1a2b730546f695`.
- `mycelium-qa6-public-webseed-restart.txt`, SHA-256
  `9661d66af687fdf2889fa90814b5fe85b0a2cf3af0393d25149890612e944aff`.
- `mycelium-qa6-public-webseed-received.SHA256SUMS`, SHA-256
  `96284aa44860db4026a5fa3070bd6b576cf08fe358247252c1c062dc3ba48b5d`.

The first loopback-bound download trial could not reach public HTTP and was
not acceptance evidence; the private-LAN-bound trial above completed. Its
temporary 4.1 GB RAM payload copies were removed after receipt retention;
durable Agora originals and public downloads remain. An unused server-side
staging ISO copy was also removed, without removing advertised artifacts.

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
