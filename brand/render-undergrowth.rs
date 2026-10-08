//! Original shader-style distance-field kernel, frozen on the CPU at build time.
//! No Shadertoy source is used. SPDX-License-Identifier: MIT OR Apache-2.0.
use std::{
    fs::File,
    io::{BufWriter, Write},
    process::ExitCode,
};

fn hash(x: i32, y: i32) -> f64 {
    let mut n = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263);
    n = (n ^ (n >> 13)).wrapping_mul(1274126177);
    (n ^ (n >> 16)) as f64 / u32::MAX as f64
}
fn noise(x: f64, y: f64) -> f64 {
    let (i, j) = (x.floor() as i32, y.floor() as i32);
    let (u, v) = (x - x.floor(), y - y.floor());
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let a = hash(i, j) * (1.0 - u) + hash(i + 1, j) * u;
    let b = hash(i, j + 1) * (1.0 - u) + hash(i + 1, j + 1) * u;
    a * (1.0 - v) + b * v
}
fn smooth(a: f64, b: f64, v: f64) -> f64 {
    let t = ((v - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
fn growth_warp(theta: f64, phase: f64) -> f64 {
    (theta * 7.0 + phase).sin() * 0.035 + (theta * 13.0 - phase).cos() * 0.013
}
fn pixel(x: f64, y: f64) -> [u8; 3] {
    let n = noise(x * 5.0, y * 5.0) * 0.6
        + noise(x * 17.0, y * 17.0) * 0.3
        + noise(x * 61.0, y * 61.0) * 0.1;
    let mut c = [16.0 + n * 10.0, 21.0 + n * 12.0, 20.0 + n * 9.0];
    // Off-center bracket-fungus rosettes: irregular growth contours, gills and
    // quiet lichen edges leave the upper-left desktop calm for windows/icons.
    for (cx, cy, scale, phase) in [
        (1.20, 0.66, 0.51, 1.2),
        (1.01, 0.87, 0.33, 3.0),
        (1.49, 0.91, 0.39, 4.8),
    ] {
        let dx = x - cx;
        let dy = (y - cy) * 1.24;
        let theta = dy.atan2(dx);
        let r = (dx * dx + dy * dy).sqrt() / scale;
        let warp = growth_warp(theta, phase) + noise(x * 23.0, y * 23.0) * 0.045;
        let d = r + warp;
        let mask = 1.0 - smooth(0.98, 1.015, d);
        let rings = (d * 53.0 + (theta * 4.0).sin() * 0.7).sin() * 0.5 + 0.5;
        let ridge = rings.powf(8.0);
        let edge = smooth(0.88, 0.98, d) * (1.0 - smooth(0.98, 1.015, d));
        let light = (0.52 - dy * 0.6 - dx * 0.2).clamp(0.2, 0.85);
        let tone = [
            28.0 + light * 35.0 + ridge * 12.0 + edge * 40.0,
            35.0 + light * 34.0 + ridge * 14.0 + edge * 41.0,
            29.0 + light * 26.0 + ridge * 10.0 + edge * 26.0,
        ];
        for i in 0..3 {
            c[i] = c[i] * (1.0 - mask) + tone[i] * mask;
        }
    }
    let vignette = 1.0 - 0.24 * ((x - 0.8).powi(2) + (y - 0.5).powi(2));
    c.map(|v| (v * vignette).clamp(0.0, 255.0) as u8)
}
// Three overlapping shelves, with the wallpaper's same organic contour field.
// Sparse, high-contrast contours are deliberately legible at favicon size.
const MARK_SHELVES: [(f64, f64, f64, f64); 3] = [
    (0.48, 0.39, 0.30, 1.2),
    (0.36, 0.61, 0.24, 3.0),
    (0.65, 0.63, 0.26, 4.8),
];
fn mark_pixel(x: f64, y: f64) -> [u8; 3] {
    let mut c = [20.0, 28.0, 24.0];
    for (cx, cy, radius, phase) in MARK_SHELVES {
        let (dx, dy) = (x - cx, (y - cy) * 1.24);
        let theta = dy.atan2(dx);
        let d = dx.hypot(dy) / radius + growth_warp(theta, phase);
        let mask = 1.0 - smooth(0.985, 1.015, d);
        let ridge = (1.0 - smooth(0.025, 0.055, (d * 4.0).fract().min(1.0 - (d * 4.0).fract())))
            * smooth(0.2, 0.3, d);
        let edge = smooth(0.92, 0.97, d);
        let tone = [
            66.0 + ridge * 60.0 + edge * 80.0,
            87.0 + ridge * 63.0 + edge * 84.0,
            63.0 + ridge * 46.0 + edge * 63.0,
        ];
        for i in 0..3 {
            c[i] = c[i] * (1.0 - mask) + tone[i] * mask;
        }
    }
    c.map(|v| v.clamp(0.0, 255.0) as u8)
}
fn write_mark_svg(out: &mut impl Write) -> std::io::Result<()> {
    writeln!(
        out,
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><title>fungOS</title><rect width="100" height="100" rx="18" fill="#141c18"/>"##
    )?;
    write_mark_shapes(out)?;
    writeln!(out, "</svg>")
}
fn write_mark_shapes(out: &mut impl Write) -> std::io::Result<()> {
    for (cx, cy, radius, phase) in MARK_SHELVES {
        for level in [1.0, 0.75, 0.5] {
            let mut path = String::new();
            for step in 0..=96 {
                let theta = step as f64 / 96.0 * std::f64::consts::TAU;
                let r = radius * (level - growth_warp(theta, phase));
                path.push_str(&format!(
                    "{} {:.2},{:.2} ",
                    if step == 0 { "M" } else { "L" },
                    (cx + r * theta.cos()) * 100.0,
                    (cy + r * theta.sin() / 1.24) * 100.0
                ));
            }
            writeln!(
                out,
                r##"<path d="{path}Z" fill="{}" stroke="{}" stroke-width="2.2" stroke-linejoin="round"/>"##,
                if level == 1.0 { "#42573f" } else { "none" },
                if level == 1.0 { "#bbcaa0" } else { "#879e70" }
            )?;
        }
    }
    Ok(())
}
// Original geometric lettering: outlines, not a system-font dependency.
// Keep the glyphs as paths so launcher rasterization is reproducible everywhere.
const WORDMARK: [&str; 6] = [
    "M20 60V14Q20 3 33 3H40M7 24H37",
    "M7 24V45Q7 60 23 60Q39 60 39 45V24M39 45V60",
    "M7 60V24M7 38Q7 24 23 24Q39 24 39 38V60",
    "M40 24V64Q40 78 24 78Q12 78 8 72M40 32Q34 24 22 24Q6 24 6 42Q6 60 22 60Q34 60 40 50",
    "M24 3C9 3 3 14 3 32C3 50 9 61 24 61C39 61 45 50 45 32C45 14 39 3 24 3Z",
    "M43 10C34 -1 8 -1 5 16C2 31 23 31 32 35C53 44 45 62 25 62C16 62 7 58 3 52",
];
fn write_app_svg(out: &mut impl Write, wide: bool) -> std::io::Result<()> {
    let (w, h) = if wide { (640, 360) } else { (512, 512) };
    writeln!(
        out,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}"><title>fungOS</title><rect width="{w}" height="{h}" fill="#141c18"/>"##
    )?;
    let mark = if wide {
        "translate(32 40) scale(2.8)"
    } else {
        "translate(106 24) scale(3)"
    };
    writeln!(out, r#"<g transform="{mark}">"#)?;
    write_mark_shapes(out)?;
    writeln!(out, "</g>")?;
    let text = if wide {
        "translate(316 144)"
    } else {
        "translate(90 366) scale(1.05)"
    };
    writeln!(
        out,
        r##"<g transform="{text}" fill="none" stroke="#e5eadd" stroke-width="6" stroke-linecap="round" stroke-linejoin="round">"##
    )?;
    for (i, path) in WORDMARK.iter().enumerate() {
        writeln!(
            out,
            r#"<path transform="translate({} 0)" d="{path}"/>"#,
            i * 52
        )?;
    }
    writeln!(out, "</g></svg>")
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<_> = std::env::args().collect();
    if args.len() == 4 && args[1] == "--app-svg" {
        let wide = match args[2].as_str() {
            "tv" => true,
            "square" => false,
            _ => return Err("app layout must be tv or square".into()),
        };
        let mut out = BufWriter::new(File::create(&args[3])?);
        write_app_svg(&mut out, wide)?;
        out.flush()?;
        return Ok(());
    }
    if args.len() == 3 && args[1] == "--mark-svg" {
        let mut out = BufWriter::new(File::create(&args[2])?);
        write_mark_svg(&mut out)?;
        out.flush()?;
        return Ok(());
    }
    let mark = args.get(1).is_some_and(|a| a == "--mark");
    if mark {
        args.remove(1);
    }
    if args.len() != 4 {
        return Err(
            "usage: render-undergrowth [--mark] WIDTH HEIGHT OUTPUT.ppm | --mark-svg OUTPUT.svg | --app-svg tv|square OUTPUT.svg"
                .into(),
        );
    }
    let w: usize = args[1].parse()?;
    let h: usize = args[2].parse()?;
    if w == 0 || h == 0 || w > 4096 || h > 4096 {
        return Err("dimensions must be 1..4096".into());
    }
    let mut out = BufWriter::new(File::create(&args[3])?);
    write!(out, "P6\n{w} {h}\n255\n")?;
    for y in 0..h {
        for x in 0..w {
            out.write_all(&if mark {
                mark_pixel(x as f64 / w as f64, y as f64 / h as f64)
            } else {
                pixel(x as f64 / h as f64, y as f64 / h as f64)
            })?;
        }
    }
    out.flush()?;
    Ok(())
}
pub fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeatable() {
        assert_eq!(pixel(1.0, 0.7), pixel(1.0, 0.7));
    }
    #[test]
    fn texture_not_flat() {
        assert_ne!(pixel(0.1, 0.1), pixel(1.2, 0.7));
    }
    #[test]
    fn noise_bounded() {
        for i in -20..20 {
            assert!((0.0..=1.0).contains(&noise(i as f64 * 0.23, 0.3)));
        }
    }
    #[test]
    fn mark_is_repeatable_and_has_contrast() {
        assert_eq!(mark_pixel(0.5, 0.4), mark_pixel(0.5, 0.4));
        assert_ne!(mark_pixel(0.5, 0.4), mark_pixel(0.01, 0.01));
    }
    #[test]
    fn vector_mark_is_finite_and_small() {
        let mut out = Vec::new();
        write_mark_svg(&mut out).unwrap();
        let svg = String::from_utf8(out).unwrap();
        assert_eq!(svg.matches("<path ").count(), 9);
        assert!(!svg.contains("NaN"));
        assert!(svg.len() < 20_000);
    }
    #[test]
    fn app_wordmarks_are_font_free_with_six_letters() {
        for wide in [true, false] {
            let mut bytes = Vec::new();
            write_app_svg(&mut bytes, wide).unwrap();
            let svg = String::from_utf8(bytes).unwrap();
            assert_eq!(svg.matches("<path ").count(), 15);
            assert!(!svg.contains("<text"));
            assert!(!svg.contains("href="));
            assert!(svg.contains(if wide {
                "width=\"640\" height=\"360\""
            } else {
                "width=\"512\" height=\"512\""
            }));
        }
    }
}
