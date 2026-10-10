//! Clone only a retained private package-qualification disk for signed APT proof.
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn run(program: &str, args: &[&str]) -> Result<String> {
    let out = Command::new(program).args(args).output()?;
    if !out.status.success() {
        return Err(format!(
            "{program}: {}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(out.stdout)?)
}
fn text(path: &Path) -> Result<&str> {
    path.to_str().ok_or_else(|| "non UTF8 path".into())
}
fn put(root: &Path, path: &str, content: &str) -> Result<()> {
    let path = root.join(path);
    fs::create_dir_all(path.parent().ok_or("missing parent")?)?;
    fs::write(path, content)?;
    Ok(())
}
fn regular(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.is_file())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 4 {
        return Err("usage: mycelium-apt-image PRIVATE_QUALIFIED_RAW_DISK NEW_OUTPUT_DIR SIGNED_REPOSITORY PROOF_BINARY".into());
    }
    if run("id", &["-u"])?.trim() != "0" {
        return Err("root required for private disposable image assembly".into());
    }
    for i in [0, 3] {
        if !regular(Path::new(&args[i])) {
            return Err("input must be regular nonsymlink file, not a device".into());
        }
    }
    let repository = Path::new(&args[2]);
    for file in [
        "SHA256SUMS",
        "dists/testing/InRelease",
        "fungos-archive-keyring.gpg",
    ] {
        if !regular(&repository.join(file)) {
            return Err("missing genuine signed repository".into());
        }
    }
    let verify = Command::new("sha256sum")
        .args(["--check", "SHA256SUMS"])
        .current_dir(repository)
        .output()?;
    if !verify.status.success() {
        return Err("signed repository inventory mismatch".into());
    }
    let out = PathBuf::from(&args[1]);
    fs::create_dir(&out)?;
    let image = out.join("disk.raw");
    run(
        "cp",
        &["--reflink=auto", "--sparse=always", &args[0], text(&image)?],
    )?;
    let device = run(
        "losetup",
        &["--find", "--show", "--partscan", text(&image)?],
    )?
    .trim()
    .to_owned();
    if !device.starts_with("/dev/loop") || !device[9..].bytes().all(|b| b.is_ascii_digit()) {
        return Err("unexpected allocated loop device".into());
    }
    let root = out.join("mount");
    fs::create_dir(&root)?;
    if let Err(error) = run("mount", &[&format!("{device}p2"), text(&root)?]) {
        run("losetup", &["-d", &device])?;
        return Err(error);
    }
    let prepared = (|| -> Result<()> {
        if !root.join("var/lib/fungos-mycelium-package-proof").is_file()
            || !root.join("usr/bin/mycelium").is_file()
            || root.join("var/lib/fungos-mycelium-apt-proof").exists()
        {
            return Err(
                "not the completed private qualification disk; refuses fleet/unqualified disk"
                    .into(),
            );
        }
        run(
            "chroot",
            &[
                text(&root)?,
                "systemctl",
                "disable",
                "fungos-mycelium-package-proof.service",
            ],
        )?;
        // No package is fabricated or installed here; the runtime engine owns
        // signed admission, staging, installation and recovery in the guest.
        let qa = root.join("opt/fungos-mycelium-apt-qa");
        fs::create_dir(&qa)?;
        run("cp", &["-a", &args[2], text(&qa.join("repository"))?])?;
        fs::copy(&args[3], qa.join("proof"))?;
        fs::set_permissions(qa.join("proof"), fs::Permissions::from_mode(0o755))?;
        for original in ["etc/apt/sources.list", "etc/apt/sources.list.d"] {
            let path = root.join(original);
            if path.exists() {
                fs::rename(
                    path,
                    root.join(format!("{original}.before-mycelium-apt-proof")),
                )?;
            }
        }
        fs::create_dir(root.join("etc/apt/sources.list.d"))?;
        put(&root, "etc/apt/sources.list", "")?;
        put(&root, "etc/apt/sources.list.d/fungos-test.sources", "Types: deb\nURIs: file:/opt/fungos-mycelium-apt-qa/repository\nSuites: testing\nComponents: main\nArchitectures: amd64\nSigned-By: /opt/fungos-mycelium-apt-qa/repository/fungos-archive-keyring.gpg\nCheck-Valid-Until: yes\n")?;
        // Ignore stale lists from the earlier OS fixture; the first real update
        // authenticates only this test authority. Private identity stays intact.
        let lists = root.join("var/lib/apt/lists");
        if lists.exists() {
            fs::rename(
                &lists,
                root.join("var/lib/apt/lists.before-mycelium-apt-proof"),
            )?;
        }
        fs::create_dir(&lists)?;
        put(&root, "usr/lib/systemd/system/fungos-mycelium-apt-proof.service", "[Unit]\nDescription=Private signed Mycelium APT qualification\nAfter=mycelium.service network-online.target\nWants=mycelium.service network-online.target\nConditionVirtualization=yes\n\n[Service]\nType=oneshot\nExecStart=/opt/fungos-mycelium-apt-qa/proof\nTimeoutStartSec=360\nUMask=0077\nStandardOutput=journal\nStandardError=journal\n\n[Install]\nWantedBy=multi-user.target\n")?;
        run(
            "chroot",
            &[
                text(&root)?,
                "systemctl",
                "enable",
                "fungos-mycelium-apt-proof.service",
            ],
        )?;
        Ok(())
    })();
    let unmounted = run("umount", &[text(&root)?]);
    if unmounted.is_ok() {
        run("losetup", &["-d", &device])?;
    }
    prepared?;
    unmounted?;
    println!("assembled private signed APT image: {}", image.display());
    Ok(())
}
#[cfg(test)]
mod tests {
    #[test]
    fn refuses_devices() {
        assert!(!super::regular(std::path::Path::new("/dev/null")));
    }
}
