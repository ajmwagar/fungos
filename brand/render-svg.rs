//! Offline rasterization of the font-free procedural brand assets.
//! SPDX-License-Identifier: MIT OR Apache-2.0
use std::{error::Error, process::ExitCode};

fn render(svg: &[u8]) -> Result<resvg::tiny_skia::Pixmap, Box<dyn Error>> {
    if svg.len() > 1024 * 1024 {
        return Err("SVG must be at most 1 MiB".into());
    }
    let tree = resvg::usvg::Tree::from_data(svg, &Default::default())?;
    let size = tree.size().to_int_size();
    if size.width() > 4096 || size.height() > 4096 {
        return Err("SVG dimensions must be at most 4096 pixels".into());
    }
    let mut pixels = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
        .ok_or("Could not allocate bounded image")?;
    resvg::render(&tree, Default::default(), &mut pixels.as_mut());
    Ok(pixels)
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: fungos-svg-render INPUT.svg OUTPUT.png".into());
    }
    if std::fs::metadata(&args[0])?.len() > 1024 * 1024 {
        return Err("SVG must be at most 1 MiB".into());
    }
    render(&std::fs::read(&args[0])?)?.save_png(&args[1])?;
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn launcher_art_is_opaque_and_correctly_sized() {
        for (svg, dimensions) in [
            (include_bytes!("fungos-tv.svg").as_slice(), (640, 360)),
            (include_bytes!("fungos-app.svg").as_slice(), (512, 512)),
        ] {
            let image = render(svg).unwrap();
            assert_eq!((image.width(), image.height()), dimensions);
            assert!(image.data().chunks_exact(4).all(|p| p[3] == 255));
        }
    }
    #[test]
    fn rejects_large_or_invalid_assets() {
        assert!(render(b"not an SVG").is_err());
        assert!(render(&vec![b' '; 1024 * 1024 + 1]).is_err());
        assert!(
            render(br#"<svg xmlns="http://www.w3.org/2000/svg" width="4097" height="1"/>"#)
                .is_err()
        );
    }
}
