# fungOS release signing

The initial OpenPGP release identity was created on 2026-10-11 UTC under
operator authorization. It is separate from Mycelium peer enrollment keys.

Primary certification fingerprint:
`FC3C04696D437DCD2F51895F0BEFDC5CF68C7CBA`

Signing subkey fingerprint:
`1F000F1503A7FD0B4A262F7EB98EF22C64EEDDC3`

The Ed25519 primary key expires 2029-10-10; the Ed25519 signing subkey expires
2027-10-11. Public exports are in `release/evidence/fungos-release-key.asc` and
`fungos-release-keyring.gpg`. The same release identity can sign image checksum
inventories and APT metadata; this does not change Mycelium's native artifact
signature format or automatically grant fleet authority.

## Private custody and recovery

The private signing home is outside Git at
`~/.local/share/fpl-release-signing/fungos`, mode 0700. The key has a randomly
generated passphrase stored in the operator's macOS Keychain under service
`dev.fungos.release-signing`, account `ajmwagar`. Never put that passphrase in
source, logs, OpenTofu state, a serving host or an object-storage bucket.

A passphrase-protected secret-key export and a revocation certificate are kept
in `~/.local/share/fpl-release-signing/fungos-recovery`, mode 0700, with files
mode 0600. **This same-machine recovery copy is not an off-device backup.**
Before routine unattended production signing, retain the recovery materials
and passphrase in an independently secured off-device backup and test recovery.
The certification key is not yet moved offline; hardware-backed custody and
CI/Fab admission are not claimed by this initial setup.

## Signing an inventory

First check all inventory entries against the selected immutable payloads.
Use fresh signature output paths; do not silently overwrite a prior release.

```sh
security find-generic-password -a ajmwagar -s dev.fungos.release-signing -w |
  gpg --homedir "$HOME/.local/share/fpl-release-signing/fungos" \
    --batch --pinentry-mode loopback --passphrase-fd 0 \
    --local-user '1F000F1503A7FD0B4A262F7EB98EF22C64EEDDC3!' \
    --armor --detach-sign --output SHA256SUMS.asc SHA256SUMS
```

The QA6 original torrent inventory has a verified detached signature in
`release/evidence/mycelium-qa6-torrents.SHA256SUMS.asc`. Webseed variants have
different torrent-file digests and must use their own signed inventory.

## Verification

Admit the complete fingerprint through an independently trusted channel before
using a downloaded key. A valid signature with an arbitrary key from the same
download server is not sufficient trust.

```sh
gpg --show-keys --with-fingerprint fungos-release-key.asc
# Compare the primary fingerprint to the independently admitted fingerprint.
gpg --dearmor --output fungos-release-keyring.gpg fungos-release-key.asc
gpgv --keyring ./fungos-release-keyring.gpg SHA256SUMS.asc SHA256SUMS
sha256sum --check SHA256SUMS
```

Verify the signature before relying on the inventory. On macOS use
`shasum -a 256 --check SHA256SUMS`. Fetch every listed file into the working
directory for a complete checksum check. APT clients use repository-specific
`Signed-By`; see `packages/README.md`. No installer enforcement, key rotation,
revocation distribution or automatic fleet trust is implied by these commands.
