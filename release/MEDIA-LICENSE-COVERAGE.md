# Qualified media source and notice coverage

This evidence applies only to ISO SHA-256
`601a631bb41cd93ef091e9b6a38b25d09bb49395a036f0bf668053dc9bde0b5c`
and its freshly installed BIOS/UEFI inventories. It is not a release approval,
an assertion that all firmware is open source, or a single license for the distro.
Each included component retains its own terms.

## Exact Debian source closure

The final inventory covers **587 unique source/version pairs**:

- Media `.deb` and `.udeb` archives, plus all three installer initrds.
- Fresh BIOS and UEFI installed status, including exact `Built-Using` and
  `Static-Built-Using` references. Source for residual config-file entries is
  conservatively included, without calling those binaries installed.
- Extracted Syslinux boot code and byte-verified Debian CD artwork.

Every DSC and all archives named in its `Checksums-Sha256` were retained and
verified for SHA-256 and length. Admission used isolated, signed APT indexes;
historical expiry relaxation applied only to explicitly pinned Debian snapshots.
No production APT configuration changed. The old text-installer-only 425-pair
inventory missed 39 graphical/Xen source pairs and 121 static/config sources.
The corrected full inventory supersedes it.

One declared static dependency, `rust-blake2=0.10.6-2`, was no longer present in
the release-window source indexes. Debian Snapshot's primary API identified
its 2023-09-26 first appearance. Its exact source was subsequently downloaded
through signed `sid` indexes pinned to `20230927T000000Z`; it was not replaced
with version `-3` or accepted solely on an unauthenticated API hash.

Private evidence root on the approved build host:
`~/.cache/fungos-release-source.20261010/`.

| Evidence | SHA-256 |
| --- | --- |
| `requirements-final-complete-debian.tsv` | `01293aa3badafe59fe1197501b060784344865420c67ad2ac615b3fff55d4458` |
| `debian-source-manifest-final-qa5.tsv` | `8ac96322d56cdfdee8f0ed6cde6969656c2121aaf252c5552512b11da6d5cf54` |
| `media-notices-final-qa5/packages.tsv` | `e50f37d675b4c5a9299672ca1f8c04c22c8cd63b1201ae7f8aa0af127b869ae5` |

`source-stage` materializes these verified source files using hardlinks in a
new private staging directory. Its published-form manifest uses relative paths,
not internal host paths. It does not include signing keys, claims, or peer state.
This is the Debian section of a future source bundle, not the complete bundle.

## Raw boot files and artwork

`media-bindings` performs actual comparisons, not filename attribution:

| Raw media | Verified origin |
| --- | --- |
| EFI shim, GRUB, and both copies inside the FAT boot image | Corresponding embedded signed Debian package payloads |
| GRUB font | `grub2=2.12-9`, `ascii.pf2` |
| All three raw kernels | Embedded signed Linux package, with underlying `linux=6.12.43-1` source |
| Four Syslinux modules | Authenticated `syslinux-common`, source `3:6.04~git20190206.bf6db5b4+dfsg1-3.1` |
| `isolinux.bin` | Same exact package body except its boot-information table, offsets 8–63 |
| ISO hybrid executable prefix | Same Syslinux `isohdpfx.bin` bytes, offsets 32–431 |
| All 17 `pics/` files | Byte-identical retained `debian-cd=3.2.2` source assets |
| Installer splash | Byte-identical `rsvg-convert` output from retained Debian Installer Ceratopsian SVG |

Syslinux `core/isolinux.asm` documents the boot-information table and
`-boot-info-table` transformation. The retained `.disk/mkisofs` recipe specifies
it. The installed upstream `xorrisofs(1)` manual documents
`-isohybrid-apm-hfsplus` replacing only the initial 32-byte mock header.
Partition geometry, GPT/APM tables, FAT directory entries, and El Torito catalog
entries are generated metadata; no whole-sector byte-identity claim is made.
The executable portions remain checked against their actual source package.

The splash's preferred source and notice are retained in authenticated
`debian-installer=20250803+deb13u1`: `build/boot/artwork/13-ceratopsian/`.
Its author is Elise Couper, copyright 2024, with GPL-2.0-or-later or
CC-BY-SA-4.0 options. The source notice records the installer-specific edits.
Rendering with `rsvg-convert 2.61.3` reproduces SHA-256
`b236addaf6932487972b83071f60356a3a9efe617dc386b3bfbf5885d1b17757`.

Debian CD's GPL-2.0-or-later notice is retained with its source. Debian's open-use
logo has separate SPI copyright and LGPL-3.0-or-later or CC-BY-SA-3.0 terms;
its upstream notice and SVG were separately retained. These are not fungOS
branding or an assertion of Debian endorsement. [Debian logo terms](https://www.debian.org/logos/).

## Package notices and firmware

`media-notices` retained package-owned copyright/common-license entries from
all **791 media archive occurrences**, without installing packages. There are
36 firmware-directory archives and all 36 have retained copyright notices.
The same firmware packages appearing in `pool/` are separate occurrences,
not additional firmware identities.

Some split packages and installer udebs have no direct copyright entry. They
retain their source package's notices, and Debian split-package documentation
may refer to another included package. A zero direct-notice count is not a
license declaration. Source archives retain the Debian packaging notices and
upstream license files; original binary archives retain their documentation
symlinks. Common-license texts accompany the source/notice bundle.

Firmware is not relabeled MIT/Apache or required to have nonexistent modifiable
source. Its package-specific binary redistribution terms and vendor notices
remain authoritative. For example, `firmware-nonfree` records excluded files
whose redistribution/source conditions were not met; those excluded blobs are
not silently pulled in from upstream linux-firmware. Only exact Debian archive
contents are included. Do not expand this evidence to an arbitrary firmware
download or a changed product/use without checking its terms.

## Combined source follow-up

The combined qa6 archive now includes Mycelium's exact owner source, dependency/
native runtime sources, reviewed notices and build scripts alongside this Debian
section. [NOTICE-PACKAGE-QUALIFICATION.md](NOTICE-PACKAGE-QUALIFICATION.md) records
its hashes, extraction verification and source coverage of fresh BIOS/UEFI installs.
Release signatures and public source availability remain required before
publication; this report does not authorize a public torrent.
