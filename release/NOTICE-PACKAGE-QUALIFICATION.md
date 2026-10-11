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
Fresh boot qualification is recorded below when complete, not inherited from qa5.

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

## Publication boundary

This slice does not authorize public upload or seeding. Release signatures,
authenticated download metadata and public availability of the matching source
bundle are separate delivery steps. The ISO remains the explicitly narrow
Mycelium-only qualification image, not a full Unibus/Canvas edge release.
