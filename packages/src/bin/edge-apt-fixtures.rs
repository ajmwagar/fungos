//! Transform owner packages into explicitly labeled qualification fixtures.
//! Not a release packager: all good versions share the supplied owner binary.
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn run(program: &str, args: &[&str]) -> Result<()> {
    let out = Command::new(program).args(args).output()?;
    if !out.status.success() {
        return Err(format!("{program}: {}", String::from_utf8_lossy(&out.stderr)).into());
    }
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err(
            "usage: edge-apt-fixtures OWNER.deb PACKAGE EXECUTABLE OUTPUT_DIRECTORY".into(),
        );
    }
    let [archive, package, binary, directory] = args.as_slice() else {
        unreachable!()
    };
    if !["canvas-linux", "unibus-core"].contains(&package.as_str()) {
        return Err("unexpected qualification package".into());
    }
    let directory = PathBuf::from(directory);
    fs::create_dir_all(&directory)?;
    for n in 1..=3 {
        let version = format!("0.1.0+aptqa{n}");
        let stage = directory.join(format!("{package}-{version}-stage"));
        fs::create_dir(&stage)?;
        run("dpkg-deb", &["-R", archive, stage.to_str().unwrap()])?;
        let control = fs::read_to_string(stage.join("DEBIAN/control"))?;
        let control = control
            .lines()
            .map(|line| {
                if line.starts_with("Version: ") {
                    format!("Version: {version}")
                } else {
                    line.into()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        fs::write(stage.join("DEBIAN/control"), control)?;
        let name = if package == "unibus-core" {
            "unibus-router"
        } else {
            "canvas-linux"
        };
        let destination = stage.join("usr/bin").join(name);
        if n == 3 {
            let source = directory.join(format!("{package}-unhealthy.rs"));
            fs::write(&source, "fn main() { std::process::exit(1); }\n")?;
            run(
                "rustc",
                &[
                    "--crate-name",
                    "unhealthy_fixture",
                    source.to_str().unwrap(),
                    "-o",
                    destination.to_str().unwrap(),
                ],
            )?;
        } else {
            fs::copy(Path::new(binary), destination)?;
        }
        run(
            "dpkg-deb",
            &[
                "--root-owner-group",
                "--build",
                stage.to_str().unwrap(),
                directory
                    .join(format!("{package}_{version}_amd64.deb"))
                    .to_str()
                    .unwrap(),
            ],
        )?;
    }
    Ok(())
}
