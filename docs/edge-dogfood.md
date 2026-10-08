# Edge dogfood: QEMU

On 8 October 2026 the persistent amd64 guest booted its native Canvas Wayland
compositor, Canvas, Unibus, Canvas edge adapter and Mycelium successfully.

## Verified behavior

The enrolled certificate and machine ID remained unchanged across reboot.
No new claim was consumed; enrollment happened previously. The guest initially
had no saved workspace. Importing the canonical Undergrowth PNG through Canvas's
owner interface and saving `dock` created one. Canvas restart and full guest
reboot retained that workspace byte-for-byte. A read-only VNC framebuffer
capture confirmed the wallpaper was actually displayed.

The production update runner completed with no new release selected. Stopping
the guest's Unibus router produced `state: drifted` with `native service is not
active`. Running the same automatic-update service restored the router and
Canvas edge adapter without changing the saved workspace. This proves
same-version repair, not a new release upgrade.

Installed native versions: Canvas 0.1.6, canvas-linux 0.1.1,
canvas-edge 0.1.0, Unibus router 0.1.1. Canvas and Unibus use the isolated
automatic test policy; compositor and adapter remain manual.

## Manual parity

Use only the disposable qualification guest. Establish authenticated SSH with
its previously verified host key; do not disable host verification. Run these
commands inside the guest, not on the build host or a fleet node:

```sh
export MYCELIUM_HOME=/var/lib/mycelium
export MYCELIUM_NO_AUTOSTART=1
export XDG_RUNTIME_DIR=/run/fungos-canvas
systemctl is-active mycelium canvas-compositor canvas canvas-edge unibus-router
/var/lib/mycelium/bin/mycelium software reconcile --dry-run --json
systemctl start mycelium-update.service
systemctl show mycelium-update.service -p Result

# Copy the canonical brand PNG to this fresh path first.
printf 'bg-image /var/lib/fungos-edge/undergrowth.png\nws-save dock\n' | canvasctl
sha256sum /var/lib/fungos-edge/workspaces.json /etc/machine-id \
  /var/lib/mycelium/pki/node.pem
systemctl restart canvas
printf 'inspect\n' | canvasctl
```

Record hashes before reboot; compare after clean guest reboot, repeat service
and owner inspection, and capture the actual framebuffer. An active compositor
alone does not establish scanout health.

For the fault test, stop only `unibus-router` in this disposable guest, require
fresh dry-run drift, start `mycelium-update.service`, then require router and
adapter readiness and unchanged workspace bytes. If repair fails, explicitly
restore the router and report failure; never leave it stopped.

## Evidence and limits

The qualification host retains the capture at
`~/.cache/fungos-clean-edge-20261007/display-20261008.ppm`.
Workspace SHA-256 after restart and reboot:
`9569ffeabfc763e4943a396753c0731bace404ab6ca02a0c5987e61be73f14ce`.

Failed-update rollback was independently qualified by the
[isolated APT test](../packages/EDGE-QUALIFICATION.md). That test uses unhealthy
executable fixtures and no fleet identity: it is not a new failed-release
injection into this visual guest or evidence of application-data migration
safety. APT and native activation must not both own the same component.
Automatic APT policy remains outside that qualification.

This check does not qualify Pi hardware, encrypted-root unlock, a fresh PXE
claim, multi-package atomic updates, GPU passthrough, or media delivery to TV.
Public screenshots and announcements require review before posting.

The later [visual rollback qualification](visual-rollback.md) separately proves
signed native Canvas/Unibus failure recovery and retained state across reboot
in an offline disposable copy of the rebuilt edge image.
