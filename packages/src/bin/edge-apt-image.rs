//! Assemble only a NEW copy of a disposable qualification disk, never a peer disk.
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn run(program: &str, args: &[&str]) -> Result<()> {
    let out = Command::new(program)
        .args(args)
        .env("DEBIAN_FRONTEND", "noninteractive")
        .env("NEEDRESTART_MODE", "l")
        .output()?;
    if !out.status.success() {
        return Err(format!(
            "{program}: {}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
        .into());
    }
    Ok(())
}
fn put(root: &Path, path: &str, content: &str) -> Result<()> {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(path, content)?;
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 6 {
        return Err("usage: edge-apt-image DISPOSABLE_BASE_IMAGE NEW_OUTPUT_DIR MYCELIUM REPOSITORY FIXTURES PROOF".into());
    }
    let [base, out, mycelium, repository, fixtures, proof] = args.as_slice() else {
        unreachable!()
    };
    let out = PathBuf::from(out);
    fs::create_dir(&out)?;
    let image = out.join("root.ext4");
    run(
        "cp",
        &[
            "--reflink=auto",
            "--sparse=always",
            base,
            image.to_str().unwrap(),
        ],
    )?;
    let root = out.join("mount");
    fs::create_dir(&root)?;
    run(
        "mount",
        &[
            "-o",
            "loop",
            image.to_str().unwrap(),
            root.to_str().unwrap(),
        ],
    )?;
    let prepared = (|| -> Result<()> {
        // Reject accidentally supplied managed-node images even though we only
        // copied the input. This tool qualifies existing test disks, not migration.
        if !root.join("opt/fungos-apt/apt-smoke").exists()
            || root.join("var/lib/mycelium/identity.json").exists()
        {
            return Err("input is not the expected disposable APT qualification image".into());
        }
        put(&root, "usr/sbin/policy-rc.d", "#!/bin/sh\nexit 101\n")?;
        run(
            "chmod",
            &["755", root.join("usr/sbin/policy-rc.d").to_str().unwrap()],
        )?;
        // Only the copied image is modified. Existing sources are retained inside
        // that image for diagnosis; cloud/host APT sources remain untouched.
        let sources = root.join("etc/apt/sources.list.d");
        if sources.exists() {
            fs::rename(
                &sources,
                root.join("etc/apt/sources.list.d.before-edge-test"),
            )?;
        }
        fs::create_dir(&sources)?;
        put(
            &root,
            "etc/apt/sources.list",
            "deb https://deb.debian.org/debian trixie main\n",
        )?;
        let resolver = root.join("etc/resolv.conf");
        if fs::symlink_metadata(&resolver).is_ok_and(|m| m.file_type().is_symlink()) {
            fs::remove_file(&resolver)?;
        }
        fs::copy("/etc/resolv.conf", &resolver)?;
        if let Ok(address) = std::env::var("FUNGOS_BUILD_DEBIAN_IP") {
            let address: std::net::IpAddr = address.parse()?;
            put(
                &root,
                "etc/hosts",
                &format!("127.0.0.1 localhost\n{address} deb.debian.org\n"),
            )?;
        }
        put(
            &root,
            "etc/nsswitch.conf",
            "passwd: files\ngroup: files\nshadow: files\nhosts: files dns\nnetworks: files\n",
        )?;
        run(
            "chroot",
            &[
                root.to_str().unwrap(),
                "apt-get",
                "update",
                "--error-on=any",
            ],
        )?;
        run(
            "chroot",
            &[
                root.to_str().unwrap(),
                "apt-get",
                "install",
                "-y",
                "--no-install-recommends",
                "libgbm1",
                "libinput10",
                "libseat1",
                "libwayland-server0",
                "libxkbcommon0",
                "adwaita-icon-theme",
            ],
        )?;
        let target = root.join("opt/fungos-edge-apt");
        fs::create_dir(&target)?;
        run(
            "cp",
            &[
                "-a",
                repository,
                target.join("repository").to_str().unwrap(),
            ],
        )?;
        fs::copy(mycelium, root.join("opt/fungos-apt/mycelium"))?;
        fs::copy(proof, root.join("opt/fungos-apt/edge-apt-proof"))?;
        run(
            "strip",
            &[root.join("opt/fungos-apt/mycelium").to_str().unwrap()],
        )?;
        for package in ["unibus-core", "canvas-linux"] {
            let archive = Path::new(fixtures).join(format!("{package}_0.1.0+aptqa1_amd64.deb"));
            let installed = target.join(archive.file_name().unwrap());
            fs::copy(archive, &installed)?;
            run(
                "chroot",
                &[
                    root.to_str().unwrap(),
                    "dpkg",
                    "--force-confold",
                    "-i",
                    &format!(
                        "/opt/fungos-edge-apt/{}",
                        installed.file_name().unwrap().to_str().unwrap()
                    ),
                ],
            )?;
        }
        // From now on APT uses only the signed local generation. No insecure
        // signature/expiry override and no package download from the LAN.
        put(&root, "etc/apt/sources.list", "")?;
        put(&root,"etc/apt/sources.list.d/fungos-test.sources","Types: deb\nURIs: file:/opt/fungos-edge-apt/repository\nSuites: testing\nComponents: main\nArchitectures: amd64\nSigned-By: /opt/fungos-edge-apt/repository/fungos-archive-keyring.gpg\nCheck-Valid-Until: yes\n")?;
        put(
            &root,
            "etc/fungos-edge-apt/token",
            "disposable-test-only-token-not-a-fleet-credential\n",
        )?;
        run(
            "chmod",
            &[
                "600",
                root.join("etc/fungos-edge-apt/token").to_str().unwrap(),
            ],
        )?;
        put(
            &root,
            "etc/unibus/hub.json",
            r#"{"installation_id":"edge-apt-test","listen":"127.0.0.1:18790","state_dir":"/var/lib/unibus","nodes":[{"producer_id":"health","token_file":"/run/credentials/unibus-router.service/health-token","zones":["health"],"capabilities":["health.probe"]}]}"#,
        )?;
        put(
            &root,
            "etc/fungos-edge-apt/health-client.json",
            r#"{"installation_id":"edge-apt-test","producer_id":"health","hub_endpoint":"127.0.0.1:18790","token_file":"/run/credentials/unibus-router.service/health-token","zones":["health"],"schema_version":2,"read_timeout_ms":100}"#,
        )?;
        put(&root,"etc/systemd/system/unibus-router.service.d/health.conf","[Service]\nLoadCredential=health-token:/etc/fungos-edge-apt/token\nEnvironment=UNIBUS_HEALTH_SOCKET=/run/unibus/health.sock\nEnvironment=UNIBUS_HEALTH_CLIENT=/etc/fungos-edge-apt/health-client.json\n")?;
        put(&root,"etc/systemd/system/canvas-compositor.service","[Unit]\nDescription=Disposable headless native Canvas qualification\n[Service]\nExecStart=/usr/bin/canvas-linux desktop-compositor headless\nEnvironment=XDG_RUNTIME_DIR=/run/canvas-apt\nEnvironment=HOME=/var/lib/canvas-apt\nRuntimeDirectory=canvas-apt\nRuntimeDirectoryMode=0700\nStateDirectory=canvas-apt\nRestart=on-failure\nRestartSec=2\n[Install]\nWantedBy=multi-user.target\n")?;
        put(
            &root,
            "var/lib/mycelium/software-services.json",
            r#"{
"unibus-core":{"unit":"unibus-router.service","timeout_secs":10,"readiness":{"kind":"unix","path":"/run/unibus/health.sock","request":"health\n","expect_prefix":"READY unibus\n"}},
"canvas-linux":{"unit":"canvas-compositor.service","timeout_secs":10,"readiness":{"kind":"unix_json","path":"/run/canvas-apt/native-control/compositor-control.sock","request":{"command":"list"},"expect":{"/ok":true}}}
}"#,
        )?;
        run(
            "chmod",
            &[
                "600",
                root.join("var/lib/mycelium/software-services.json")
                    .to_str()
                    .unwrap(),
            ],
        )?;
        put(&root,"usr/lib/systemd/system/fungos-qemu-proof.service","[Unit]\nDescription=Disposable Mycelium edge APT update proof\nAfter=unibus-router.service canvas-compositor.service\nWants=unibus-router.service canvas-compositor.service\n[Service]\nType=oneshot\nEnvironment=MYCELIUM_HOME=/var/lib/mycelium\nExecStart=/opt/fungos-apt/edge-apt-proof\nStandardOutput=journal+console\nStandardError=journal+console\nTimeoutStartSec=300\n[Install]\nWantedBy=multi-user.target\n")?;
        run(
            "chroot",
            &[
                root.to_str().unwrap(),
                "systemctl",
                "enable",
                "unibus-router",
                "canvas-compositor",
            ],
        )?;
        // Existing proof stage marker belongs to the old qualification; the new
        // harness has a separate marker and does not import its result.
        Ok(())
    })();
    run("umount", &[root.to_str().unwrap()])?;
    prepared?;
    println!("prepared {}", image.display());
    Ok(())
}
