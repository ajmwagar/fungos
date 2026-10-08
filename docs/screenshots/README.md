# Real QEMU desktop captures

These unmodified 1280×800 PNGs show the native Canvas Wayland/KMS compositor
running in the `fungos-edge-qemu-01` development guest. The applications in
the workspace capture are real Weston Wayland terminals. No feature UI or
window content was composited into the images.

| Capture | UTC time, 6 October 2026 | PNG SHA-256 |
| --- | --- | --- |
| `fungos-desktop.png` | 21:34:09 | `1f865286f051e76aab98f127ed696ef98368616edc52f8d5938e465fff8a3e59` |
| `fungos-workspace.png` | 21:37:54 | `5c14533f75c5e2dbd3f917b5edbdc2f9cc9aac874d1bf10d8dff711e5db65edc` |

Source revision: `da41019`. Capture transport: read-only loopback VNC.
The frozen Undergrowth background is original procedural artwork authored
for fungOS, not copied Shadertoy source. Cursor assets in the guest come from
Debian's `xcursor-themes`; their license notices remain in the guest's
`/usr/share/doc/xcursor-themes/copyright`.

Original public assets and notices:
[ajmwagar/fungos-web](https://github.com/ajmwagar/fungos-web/blob/main/ASSET-NOTICES.md).
