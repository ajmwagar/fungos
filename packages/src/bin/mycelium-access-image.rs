//! Clone only a completed private package QA disk for isolated access checks.
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    process::Command,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn run(program: &str, args: &[&str]) -> Result<String> {
    let out = Command::new(program).args(args).output()?;
    if !out.status.success() {
        return Err(format!("{program}: {}", String::from_utf8_lossy(&out.stderr)).into());
    }
    Ok(String::from_utf8(out.stdout)?)
}
fn put(root: &Path, path: &str, data: &str) -> Result<()> {
    let path = root.join(path);
    fs::create_dir_all(path.parent().ok_or("no parent")?)?;
    fs::write(path, data)?;
    Ok(())
}
fn regular(path: &str) -> bool {
    fs::symlink_metadata(path).is_ok_and(|v| v.is_file())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 && args.len() != 4 {
        return Err("usage: mycelium-access-image PRIVATE_QUALIFIED_RAW_DISK NEW_OUTPUT_DIR ACCESS_PROOF_BINARY [GENUINE_OWNER_DEB]".into());
    }
    if run("id", &["-u"])?.trim() != "0"
        || !regular(&args[0])
        || !regular(&args[2])
        || args.get(3).is_some_and(|p| !regular(p))
    {
        return Err("root and regular nonsymlink inputs required; never host disks".into());
    }
    let out = PathBuf::from(&args[1]);
    fs::create_dir(&out)?;
    let disk = out.join("disk.raw");
    let root = out.join("mount");
    fs::create_dir(&root)?;
    run(
        "qemu-img",
        &[
            "convert",
            "-f",
            "raw",
            "-O",
            "raw",
            &args[0],
            disk.to_str().ok_or("UTF8")?,
        ],
    )?;
    let device = run(
        "losetup",
        &[
            "--find",
            "--show",
            "--partscan",
            disk.to_str().ok_or("UTF8")?,
        ],
    )?
    .trim()
    .to_owned();
    if !device.starts_with("/dev/loop") || !device[9..].bytes().all(|v| v.is_ascii_digit()) {
        return Err("unexpected loop device".into());
    }
    if let Err(error) = run(
        "mount",
        &[&format!("{device}p2"), root.to_str().ok_or("UTF8")?],
    ) {
        run("losetup", &["-d", &device])?;
        return Err(error);
    }
    let prepared = (|| -> Result<()> {
        if !root.join("var/lib/fungos-mycelium-package-proof").is_file()
            || !root.join("usr/bin/mycelium").is_file()
            || root.join("var/lib/fungos-access-proof").exists()
        {
            return Err("not an unused private qualified package disk".into());
        }
        let binary = run(
            "sha256sum",
            &[root.join("usr/bin/mycelium").to_str().ok_or("UTF8")?],
        )?;
        for ancestor in [&root, &root.join("etc"), &root.join("etc/ssh")] {
            let metadata = fs::metadata(ancestor)?;
            if metadata.uid() != 0 || metadata.permissions().mode() & 0o022 != 0 {
                return Err("input OS SSH ancestors are not root-owned/non-writable; fix installer, never patch fixture".into());
            }
        }
        let mut qualified_binary = binary.split_whitespace().next()
            == Some(fungos_apt_repository::QUALIFIED_MYCELIUM_AMD64_SHA256);
        let path = root.join("opt/fungos-access-proof/proof");
        fs::create_dir_all(path.parent().ok_or("parent")?)?;
        fs::copy(&args[2], &path)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
        if let Some(package) = args.get(3) {
            if run("dpkg-deb", &["-f", package, "Package"])?.trim() != "mycelium"
                || run("dpkg-deb", &["-f", package, "Architecture"])?.trim() != "amd64"
            {
                return Err("candidate is not the genuine amd64 owner package".into());
            }
            let extracted = out.join("candidate-payload");
            run(
                "dpkg-deb",
                &["-x", package, extracted.to_str().ok_or("UTF8")?],
            )?;
            let exec_sha = run(
                "sha256sum",
                &[extracted.join("usr/bin/mycelium").to_str().ok_or("UTF8")?],
            )?
            .split_whitespace()
            .next()
            .ok_or("missing binary digest")?
            .to_owned();
            qualified_binary |= binary.split_whitespace().next() == Some(exec_sha.as_str());
            let package_sha = run("sha256sum", &[package])?
                .split_whitespace()
                .next()
                .ok_or("missing package digest")?
                .to_owned();
            fs::copy(package, root.join("opt/fungos-access-proof/candidate.deb"))?;
            put(
                &root,
                "opt/fungos-access-proof/candidate.sha256",
                &format!("{package_sha}\n"),
            )?;
            put(
                &root,
                "opt/fungos-access-proof/candidate-binary.sha256",
                &format!("{exec_sha}\n"),
            )?;
        }
        if !qualified_binary {
            return Err("input executable differs from qualified historical or pinned genuine candidate payload".into());
        }
        run(
            "chroot",
            &[
                root.to_str().ok_or("UTF8")?,
                "systemctl",
                "disable",
                "fungos-mycelium-package-proof.service",
            ],
        )?;
        put(&root,"usr/lib/systemd/system/fungos-access-proof.service","[Unit]\nDescription=Private personal certificate access qualification\nAfter=network-online.target mycelium.service ssh.service\nWants=network-online.target mycelium.service ssh.service\nConditionVirtualization=yes\n[Service]\nType=oneshot\nExecStart=/opt/fungos-access-proof/proof\nTimeoutStartSec=180\nUMask=0077\nStandardOutput=journal\nStandardError=journal\n[Install]\nWantedBy=multi-user.target\n")?;
        run(
            "chroot",
            &[
                root.to_str().ok_or("UTF8")?,
                "systemctl",
                "enable",
                "fungos-access-proof.service",
            ],
        )?;
        Ok(())
    })();
    let unmounted = run("umount", &[root.to_str().ok_or("UTF8")?]);
    if unmounted.is_ok() {
        run("losetup", &["-d", &device])?;
    }
    prepared?;
    unmounted?;
    println!(
        "assembled private access qualification clone: {}",
        disk.display()
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn device_input_denied() {
        assert!(!regular("/dev/null"));
    }
}
