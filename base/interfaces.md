# Runtime interfaces

These contracts keep the base image independent from Mycelium internals and
from any particular update system.

## First contact

`fungos-first-contact` discovers exactly one executable file in
`/usr/lib/fungos/first-contact.d/` and invokes it with one argument: the path
to `/etc/fungos/first-contact.env`. The optional environment file is owned by
the provisioner and must contain only `KEY=VALUE` records. The adapter owns
validation, transport, retries, and durable enrollment state. Exit zero means
enrollment is complete; any other status leaves the oneshot unit failed and
eligible for retry on the next boot or manual restart.

No token, endpoint, certificate, or Mycelium package is embedded in the base
image. A Mycelium adapter consumes an owner-only claim file with `mycelium
setup --claim-file PATH`; successful redemption removes it. Genesis must
deliver that file through a protected, single-use bootstrap envelope rather
than TFTP, a public iPXE script, kernel arguments, or an environment variable.

## Signed update verification

`fungos-verify-update ARTIFACT MANIFEST SIGNATURE` discovers exactly one
executable verifier in `/usr/lib/fungos/update-verifiers.d/` and passes those
three paths unchanged. Exit zero authorizes a caller to stage the artifact.
Any other result must prevent installation. The verifier owns the signature
format, trusted-key policy, expiry checks, rollback protection, and manifest
schema. The runner never installs an artifact itself.

Both directories intentionally use a single-adapter rule. Ambiguous policy is
an error, not implicit ordering.
