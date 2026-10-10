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
ambiguous mapping fails. Kernel-generated udebs still need matching authenticated
binary Packages metadata, not a guessed source name. Signed-kernel build recipes
alone are not a substitute for the corresponding kernel implementation sources.

Owner software needs its exact committed source, patches and build scripts plus
locked dependency sources and notices. `cargo vendor --locked --versioned-dirs`
collects registry sources, but does not by itself collect Rust standard-library,
musl, compiler-runtime or every bundled native dependency's corresponding source.
These inputs and redistribution notices must be checked against the actual
final executable. A GitHub repository archive alone is insufficient.

Current preparation is explicitly incomplete: source resolution, complete
bootloader/firmware coverage and the owner toolchain/native-source closure remain
unqualified. Actual archives are retained privately on the approved build host;
no public source bundle or release torrent is approved by this note.
