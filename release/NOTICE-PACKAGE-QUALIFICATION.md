# Mycelium notice-bearing package qualification

The engineering notice review is complete for the conservative scope in
[MYCELIUM-NOTICE-REVIEW.NOTICE](MYCELIUM-NOTICE-REVIEW.NOTICE). This is not legal
certification, release signing or publication approval. The earlier qa5 package
is not made publishable retroactively.

## Actual qa6 package

The clean `f0980fba00412ed0772effc9a80d2d2446f3108b` owner packager produced
`mycelium_0.1.0+qualification6_amd64.deb`, 13,744,378 bytes, SHA-256
`c6b957afe046cec084b89383cabf1b69b7ad23ae5baf8711667659c03fffa023`.
Its executable remains byte-identical to the qualified qa5 executable, SHA-256
`60664012ba7b7b3a7c3ece05975de582caa9b6cc207a4f1b3ce535f17d3c2fa8`,
from source `10a3c18ebea8b9f3f7be981657957ac10a5a2f16`. The package records
executable and packager revisions separately. Its unit and first-contact adapter
also compare byte-identical to qa5. There are no maintainer scripts.

The package embeds **12,026 nonempty notice files** and binding/inventory metadata
under `/usr/share/doc/mycelium/third-party-notices`. Every embedded notice was
independently checked after extraction. Inventory SHA-256:
`ce8a1908e011ad3e75e7dd06acec3566147e645ccfe2b256063930d136c9c0dc`.
Every one of the 500 exact registry versions has retained notice evidence.

This is a source-closure notice superset, not a claimed minimal linked-code graph.
Build-only and inactive components are explicitly distinguished by the review;
additional compiler notices do not relicense the application. Both initial and
later exact upstream supplement collections are included: the initial collection
contains `valuable` and Android verifier notices absent from the later collection.
The enum-assoc and pharos declaration-only exceptions are explicitly reviewed,
with original declarations and labelled standard texts, not invented attribution.

The ten generic packager tests passed in a clean archive. Its real incomplete-input
negative gate failed before output creation; the actual reviewed input succeeded.
The notice collector's three focused tests passed, including batched hashing
across its 64-file boundary, spaces in filenames, changed bytes and missing files.

## Actual refreshed media and source inputs

Clean fungOS source `41ea743063f0bd9d7018f08227043bef47b03ba7` built the new
Mycelium-only ISO, SHA-256
`a77efddc847665307fc242bab1834522c7de3c838f486054ece9ada31a29c809`.
The component bundle verifier passed; every ISO file checksum passed on an actual
read-only mounted image. Its embedded package matches the qa6 package digest.
Fresh boot qualification below applies to these actual bytes, not inherited qa5 media.

The complete staged source bundle combines 587 exact Debian closures, authenticated
Ubuntu native source closures, clean owner/packager/fungOS sources, all 500 vendor
sources, actual Rust and Zig runtime sources, original logo sources and raw media
notices. All **61,279 regular-file hashes** were independently checked against
`SHA256SUMS`, SHA-256
`da39ebb4d268053075827d0f6c17b25d06727d17036ff0154efc6bb5090f389b`.
That inventory excludes only its own root checksum file, not nested inventories.
Original upstream sources and fixtures are retained; private enrolled guest disks,
operator credentials, authority keys and runtime claims are not bundle inputs.

Private retained artifact/evidence root on Agora:
`/home/ajmwagar/.cache/fungos-notice-qa6.lzDJzr`.
The package is in `/home/ajmwagar/.cache/mycelium-notice-qa6-artifacts`.
The source tree is staged using hardlinks to verified retained source inputs;
do not edit it or its source caches after hashing.

## Fresh BIOS and UEFI qualification

Both guarded fresh 8 GiB installs emitted `FUNGOS_INSTALL_COMPLETE`. Both then
booted natively from their installed disks alone to `fungos login:`; the harnesses
exited zero. Read-only inspection verified root-owned `0755` ancestors `/`, `/etc`,
`/etc/ssh` and executable, exact installed executable/inventory digests, all
12,026 installed notice hashes, no enrollment environment and no runtime claim
configuration. Both disks passed `qemu-img check` before cleanup.

The actual installed dpkg statuses were exported and passed the existing exact
source resolver. All installed Debian source/version pairs and static references
are covered by the retained closure; Mycelium is covered by its separately bound
owner source. No new missing source versions were discovered.

| Evidence | SHA-256 |
| --- | --- |
| UEFI install log | `66e07d234e5b2915fb71842ef08745d926c7bea11e5cc1bcd4a2af79098cd3a7` |
| UEFI native boot log | `09956bb764cf49ab7684c0307890147a152e51e764249b118bd16b1c112e1a09` |
| UEFI dpkg status | `c0085425a3bc699cc087058f289af19c177f457700937c08f6450942d1744b28` |
| BIOS install log | `ebbed026ed7778b5c0038a98e77044405c5df09585ebc6566b10f6a7f32f7c4d` |
| BIOS native boot log | `cb29f7433b6aebbd8a9cda0d9191616e0b1e57940979e4b8640da49313e38b91` |
| BIOS dpkg status | `dba30eb08c08bbbde381813e8b1585e06e7a19968888ae293908a3d19abc4a3c` |

This notice-only follow-up did not rerun private claim, SSH or signed-APT lifecycle
tests. Those prior results concern the unchanged executable; they are not presented
as new qa6 lifecycle runs. Full interactive installation and other architectures
remain outside this narrow qualification. The known Debian AF_VSOCK generator
warning is not relabelled as a clean whole-OS security/compliance assessment.

## Complete compressed source artifact

`fungos-mycelium-qa6-sources.tar.zst` is **3,271,330,118 bytes**, SHA-256
`79300cc84ffbe19eb547f41cdfa70764e9207b675a162c17ce385e3a39a57600`.
It is durably retained alongside the ISO under the Agora evidence root. It passed
Zstandard integrity testing, full extraction, verification of all 61,279 extracted
file hashes, and a fresh digest check after copying from RAM-backed staging to disk.

The archive was generated on Linux with:

```sh
tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner \
  -C sources -cf - . | zstd -T2 -3 --no-progress -o NEW_SOURCES.tar.zst
```

Keep inputs quiescent and use an absent output path. Reverify the archive and the
extracted `SHA256SUMS`; successful compression alone is not source coverage.

Only newly created disposable disks, unpacked harness initrds, redundant notice
extractions and the collector's development target were removed for space. Logs,
source/checksum inventories, original sources, final reviewed notice input,
compressed source archive, package and ISO remain. Removed notice copies are
recoverable from the final notice input and original sources; dummy disks are
recreatable with the harness. Inspection mounts/NBD connections were detached
and the temporarily loaded NBD module was removed. No production identity changed.
Interrupted/space-constrained Neo artifact copies were removed; final artifacts
remain on Agora, not in Neo's nearly-full cache.

## Publication boundary

This slice does not authorize public upload or seeding. Release signatures,
authenticated download metadata and public availability of the matching source
bundle are separate delivery steps. The ISO remains the explicitly narrow
Mycelium-only qualification image, not a full Unibus/Canvas edge release.
