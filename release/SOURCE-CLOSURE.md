# Corresponding-source preparation

The release tool prepares torrents; it does not establish that the supplied
source archive satisfies every included package's source-distribution duties.
Do not publish a release merely because `fungos-release` produced a manifest.

The private October 10 qualification inputs include a Debian 13.1 netinst ISO,
its installer initrd and both BIOS/UEFI installed-package inventories. The
installed system alone is not the media source inventory: installer udebs,
bootloader files and firmware must also be covered. No private qualified guest
disk is a release artifact.

`source-closure inventory` reads the installed TSV, every media deb/udeb and
the initrd dpkg status. `source-closure fetch` retains exact source archives
through isolated, signature-checked APT using the selected Debian snapshots.
It writes incremental `coverage.tsv` and `missing.tsv`, never a completeness
marker. Historical snapshot expiry is disabled for this source retrieval only;
signature and hash checks remain mandatory. Production update policy is unchanged.

The compact initrd status sometimes omits Source. `resolve-initrd` resolves
binary names against authenticated source-index Binary fields; an unresolved or
ambiguous mapping fails. Folded Binary fields must be read in full, including
kernel-generated udebs; otherwise matching authenticated binary Packages metadata
is needed, not a guessed source name. Signed-kernel build recipes
alone are not a substitute for the corresponding kernel implementation sources.

Owner software needs its exact committed source, patches and build scripts plus
locked dependency sources and notices. `cargo vendor --locked --versioned-dirs`
collects registry sources, but does not by itself collect Rust standard-library,
musl, compiler-runtime or every bundled native dependency's corresponding source.
These inputs and redistribution notices must be checked against the actual
final executable. A GitHub repository archive alone is insufficient.

The observed installer pseudo-package `cdrom-isolinux-20250803+deb13u1` is
explicitly audited, not treated as a Debian source version. Authenticated Debian
source `debian-installer=20250803+deb13u1` contains `build/Makefile`609–611,
which emits targetstring-BUILD_DATE, and `debian/rules`4–5,51, which derives
DATE from its changelog version and supplies BUILD_DATE. The resolver accepts
only this exact audited marker and still requires its exact signed source-index
entry. Future daily/custom markers fail until separately audited.

Retained source archive SHA-256:
`1ebd606d81ec64fda00f8f7c9585df6bf3b2f99135160dc67546ab63e7caf5d0`;
DSC SHA-256:
`c37e94ec74ce8962c1ff7d3c0e0c406a197399bfa5157827dd2df798a992c765`.

For the retained older qualification ISO and installed inventory, corrected
package/initrd requirements contain 425 unique source/version pairs. All 425
were actually downloaded through isolated signed APT: 422 in the initial
collection, then exact Debian Installer, bogl and libdebian-installer archives.
The initial unresolved-binary list is not evidence that those sources are absent.
Its false missing entries are superseded by the signed-index identity resolution.

Current release preparation is explicitly incomplete: the final rebuilt image's
inventory must be reconciled, raw bootloader/firmware coverage and notices audited,
and the owner toolchain/native-source closure completed. Actual archives are
retained privately on the approved build host;
no public source bundle or release torrent is approved by this note.
