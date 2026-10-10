//! Copy only a qualification disk; stage genuine packages and a private proof.
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
fn put(root: &Path, relative: &str, content: &str) -> Result<()> {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().ok_or("parent missing")?)?;
    fs::write(path, content)?;
    Ok(())
}
fn regular(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok_and(|m| m.is_file())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 5 {
        return Err("usage: mycelium-package-image QUALIFICATION_QCOW2 NEW_OUTPUT_DIR PACKAGE_A PACKAGE_B PROOF_BINARY".into());
    }
    if run("id", &["-u"])?.trim() != "0" {
        return Err("root required for disposable image assembly".into());
    }
    for index in [0, 2, 3, 4] {
        if !regular(Path::new(&args[index])) {
            return Err("inputs must be regular nonsymlink files, never host devices".into());
        }
    }
    let out = PathBuf::from(&args[1]);
    fs::create_dir(&out)?;
    let image = out.join("disk.raw");
    run(
        "qemu-img",
        &[
            "convert",
            "-f",
            "qcow2",
            "-O",
            "raw",
            &args[0],
            text(&image)?,
        ],
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
    let mounted = run("mount", &[&format!("{device}p2"), text(&root)?]);
    if let Err(error) = mounted {
        run("losetup", &["-d", &device])?;
        return Err(error);
    }
    let prepared = (|| -> Result<()> {
        if !root
            .join("usr/share/fungos/installer/installed.tsv")
            .is_file()
            || root.join("var/lib/mycelium").exists()
            || root.join("usr/bin/mycelium").exists()
        {
            return Err("input is not an unenrolled fungOS installer qualification disk".into());
        }
        // This private clone gets fresh local host facts, never an identity copy.
        fs::write(root.join("etc/machine-id"), b"")?;
        if fs::symlink_metadata(root.join("var/lib/dbus/machine-id")).is_ok() {
            fs::remove_file(root.join("var/lib/dbus/machine-id"))?;
        }
        for file in fs::read_dir(root.join("etc/ssh"))? {
            let file = file?;
            if file.file_name().to_string_lossy().starts_with("ssh_host_") {
                fs::remove_file(file.path())?;
            }
        }
        let qa = root.join("opt/fungos-mycelium-package-qa");
        fs::create_dir_all(&qa)?;
        for (input, name) in [(&args[2], "a"), (&args[3], "b")] {
            let digest = run("sha256sum", &[input])?
                .split_whitespace()
                .next()
                .ok_or("missing digest")?
                .to_owned();
            fs::copy(input, qa.join(format!("{name}.deb")))?;
            fs::write(qa.join(format!("{name}.sha256")), format!("{digest}\n"))?;
        }
        fs::copy(&args[4], qa.join("proof"))?;
        fs::set_permissions(qa.join("proof"), fs::Permissions::from_mode(0o755))?;
        fs::create_dir_all(root.join("var/log/journal"))?;
        put(
            &root,
            "etc/systemd/journald.conf.d/90-package-proof.conf",
            "[Journal]\nStorage=persistent\n",
        )?;
        // Satisfy the real online dependency without any NIC/backend or route.
        put(
            &root,
            "etc/systemd/network/10-package-proof.netdev",
            "[NetDev]\nName=qa0\nKind=dummy\n",
        )?;
        put(&root, "etc/systemd/network/10-package-proof.network", "[Match]\nName=qa0\n[Link]\nRequiredForOnline=yes\n[Network]\nAddress=192.0.2.1/32\nLinkLocalAddressing=no\nConfigureWithoutCarrier=yes\n")?;
        put(&root, "usr/lib/systemd/system/fungos-mycelium-package-proof.service", "[Unit]\nDescription=Private disposable Mycelium package qualification\nAfter=network-online.target\nWants=network-online.target\nConditionVirtualization=yes\n\n[Service]\nType=oneshot\nExecStart=/opt/fungos-mycelium-package-qa/proof\nTimeoutStartSec=240\nUMask=0077\nStandardOutput=journal\nStandardError=journal\n\n[Install]\nWantedBy=multi-user.target\n")?;
        run(
            "chroot",
            &[
                text(&root)?,
                "systemctl",
                "enable",
                "fungos-mycelium-package-proof.service",
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
    println!(
        "assembled private package qualification image: {}",
        image.display()
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_device_and_symlink_inputs() {
        assert!(!regular(Path::new("/dev/null")));
    }
}
