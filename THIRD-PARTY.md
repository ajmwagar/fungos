# Licensing boundary

`MIT OR Apache-2.0` applies to the first-party source in this repository, not
every byte of a generated operating-system image.

- Linux retains its GPL-2.0-only licensing and applicable exceptions.
- Debian packages retain the licenses recorded by their upstream authors and
  Debian package copyright metadata.
- Raspberry Pi and other board firmware must be evaluated under their own
  redistribution terms; none is bundled in this initial source import.
- Mycelium, Unibus, Canvas, Shroud, UMIE, Yggdrasil and other optional software
  retain the licenses of their owning repositories. Inclusion in an image is
  not a license change.

The base builder preserves Debian package copyright files under
`/usr/share/doc/*/copyright`. A distributor must also satisfy each included
component's applicable source, notice and redistribution requirements. This
document is an inventory boundary, not a completed release-compliance audit.

The initial public repository does not ship kernels, firmware blobs, application
binaries or generated images. Before publishing downloadable images, record
the actual package versions, licenses, source locations and local patches for
each release. Keep optional AGPL applications clearly identified; they are not
silently relicensed by the permissive license on the distribution tooling.

Screenshot-specific provenance and attribution live in
[`docs/screenshots/README.md`](docs/screenshots/README.md).
