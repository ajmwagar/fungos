# Undergrowth

The fungOS mark is three overlapping bracket-fungus shelves. It uses the same
original distance-field growth contours as the Undergrowth desktop wallpaper.
`fungos.svg` is the simplified small-size mark; `fungos-icon.png` is the shaded
512 × 512 avatar. `undergrowth.png` is the 1600 × 1000 wallpaper.

The sole procedural source lives in [Mycelium's theme renderer](https://github.com/ajmwagar/mycelium/blob/master/fungOS/edge/theme/render-undergrowth.rs).
No third-party shader, font, generated illustration or Shadertoy source is used.
Render once, retain the image; there is no animated wallpaper workload.

```sh
rustc --test /path/to/mycelium/fungOS/edge/theme/render-undergrowth.rs -o /tmp/fungos-art-tests
/tmp/fungos-art-tests
rustc -O /path/to/mycelium/fungOS/edge/theme/render-undergrowth.rs -o /tmp/fungos-art
/tmp/fungos-art --mark-svg brand/fungos.svg
/tmp/fungos-art --mark 512 512 /tmp/fungos-mark.ppm
/tmp/fungos-art 1600 1000 /tmp/undergrowth.ppm
ffmpeg -v error -y -i /tmp/fungos-mark.ppm -frames:v 1 brand/fungos-icon.png
ffmpeg -v error -y -i /tmp/undergrowth.ppm -frames:v 1 brand/undergrowth.png
```

Code and artwork: MIT OR Apache-2.0. Trademark rights are separate; see
[TRADEMARKS.md](../TRADEMARKS.md). Consumers may bundle generated assets without
depending on Mycelium at runtime. Regenerate from the same source, not a copied kernel.
