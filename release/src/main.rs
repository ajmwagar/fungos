//! Prepare explicit, immutable release files using upstream Transmission tools.
//! This does not publish, sign, seed, or certify an image as safe to distribute.
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

type Result<T> = std::result::Result<T, String>;

const USAGE: &str = "fungos-release IMAGE EXPECTED_SHA256 SOURCES LICENSES PACKAGES BUILD BOOT_EVIDENCE NEW_OUTPUT_DIRECTORY [HTTPS_WEBSEED_BASE]\nRequires sha256sum (Linux) or shasum (macOS), transmission-create and transmission-show.\nSOURCES must be the complete corresponding-source archive; the remaining inputs are operator-reviewed nonempty files. No signing, publishing or seeding occurs.";

struct Inputs {
    files: Vec<PathBuf>,
    expected: String,
    out: PathBuf,
    webseed: Option<String>,
}

fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 160
        && name != "."
        && name != ".."
        && !name.starts_with('.')
        && !name.starts_with('-')
        && name
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"._-".contains(&c))
}

fn file_name(path: &Path) -> Result<&str> {
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("invalid release filename")?;
    if !safe_name(name) {
        return Err(
            "release filename must use portable ASCII letters, digits, '.', '_' or '-'".into(),
        );
    }
    Ok(name)
}

fn digest_valid(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn webseed_valid(s: &str) -> bool {
    let Some(rest) = s.strip_prefix("https://") else {
        return false;
    };
    !rest.is_empty()
        && !rest.starts_with('/')
        && !rest.contains(['@', '?', '#', '\\'])
        && rest.bytes().all(|b| b.is_ascii_graphic())
        && !rest.split('/').any(|s| s == "." || s == "..")
}

fn parse(args: Vec<String>) -> Result<Inputs> {
    if !(args.len() == 8 || args.len() == 9) {
        return Err(USAGE.into());
    }
    if !digest_valid(&args[1]) {
        return Err("expected image SHA-256 must be 64 lowercase hex characters".into());
    }
    let mut files = vec![PathBuf::from(&args[0])];
    files.extend(args[2..7].iter().map(PathBuf::from));
    for file in &files {
        file_name(file)?;
        let meta = fs::symlink_metadata(file).map_err(|e| format!("cannot inspect input: {e}"))?;
        if !meta.is_file() || meta.len() == 0 {
            return Err(
                "inputs must be nonempty regular files, not directories or symlinks".into(),
            );
        }
    }
    let names = files
        .iter()
        .map(|p| file_name(p))
        .collect::<Result<Vec<_>>>()?;
    for (i, name) in names.iter().enumerate() {
        if names[..i].contains(name)
            || ["SHA256SUMS", "PREPARED", "PREPARING"].contains(name)
            || names
                .iter()
                .any(|other| *other == format!("{name}.torrent"))
        {
            return Err("release filenames collide".into());
        }
    }
    let out = PathBuf::from(&args[7]);
    if out.exists() {
        return Err("output directory already exists; never overwrite a release".into());
    }
    let webseed = args.get(8).cloned();
    if webseed.as_ref().is_some_and(|s| !webseed_valid(s)) {
        return Err(
            "webseed must be an HTTPS base URL without credentials, query or fragment".into(),
        );
    }
    Ok(Inputs {
        files,
        expected: args[1].clone(),
        out,
        webseed,
    })
}

fn run(program: &str, args: &[String]) -> Result<String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("cannot execute {program}: {e}"))?;
    if !output.status.success() {
        return Err(format!("{program} failed ({})", output.status));
    }
    String::from_utf8(output.stdout).map_err(|_| format!("invalid output from {program}"))
}

fn sha256(path: &Path) -> Result<String> {
    let path = path
        .canonicalize()
        .map_err(|e| format!("cannot resolve checksum input: {e}"))?;
    let path = path.to_str().ok_or("non-UTF8 checksum path")?.to_owned();
    let output = if cfg!(target_os = "macos") {
        run("shasum", &["-a".into(), "256".into(), path])?
    } else {
        run("sha256sum", &[path])?
    };
    let digest = output
        .split_whitespace()
        .next()
        .ok_or("empty checksum output")?;
    if !digest_valid(digest) {
        return Err("invalid SHA-256 tool output".into());
    }
    Ok(digest.into())
}

fn torrent_args(file: &Path, torrent: &Path, webseed: Option<&str>) -> Result<Vec<String>> {
    let mut args = vec![
        "--anonymize".into(),
        "--piecesize".into(),
        "1024".into(),
        "--outfile".into(),
        torrent.to_str().ok_or("non-UTF8 torrent path")?.into(),
    ];
    if let Some(base) = webseed {
        args.extend([
            "--webseed".into(),
            format!("{}/{}", base.trim_end_matches('/'), file_name(file)?),
        ]);
    }
    args.push(file.to_str().ok_or("non-UTF8 artifact path")?.into());
    Ok(args)
}

fn prepare(inputs: Inputs) -> Result<()> {
    if sha256(&inputs.files[0])? != inputs.expected {
        return Err("image digest does not match boot-tested candidate".into());
    }
    // Fail before copying multi-GB files if the upstream tooling is unavailable.
    run("transmission-create", &["--version".into()])?;
    run("transmission-show", &["--version".into()])?;
    fs::create_dir(&inputs.out)
        .map_err(|e| format!("cannot create fresh release directory: {e}"))?;
    fs::write(
        inputs.out.join("PREPARING"),
        b"Incomplete preparation. Do not publish or seed.\n",
    )
    .map_err(|e| e.to_string())?;
    let out = inputs.out.canonicalize().map_err(|e| e.to_string())?;
    let mut copied = Vec::new();
    for input in &inputs.files {
        let destination = out.join(file_name(input)?);
        fs::copy(input, &destination).map_err(|e| format!("cannot copy artifact: {e}"))?;
        let digest = sha256(&destination)?;
        // Detect ordinary source changes/copy corruption; build inputs must remain quiescent.
        if digest != sha256(input)? {
            return Err("input changed during preparation or copy digest differs".into());
        }
        copied.push(destination);
    }
    if sha256(&copied[0])? != inputs.expected {
        return Err("staged image changed from tested digest".into());
    }
    // Only the explicitly supplied image and corresponding sources enter torrents.
    for file in copied.iter().take(2).cloned().collect::<Vec<_>>() {
        let torrent = out.join(format!("{}.torrent", file_name(&file)?));
        run(
            "transmission-create",
            &torrent_args(&file, &torrent, inputs.webseed.as_deref())?,
        )?;
        run(
            "transmission-show",
            &[torrent.to_str().ok_or("non-UTF8 torrent path")?.into()],
        )?;
        if fs::metadata(&torrent).map_err(|e| e.to_string())?.len() == 0 {
            return Err("empty torrent output".into());
        }
        copied.push(torrent);
    }
    let mut sums = String::new();
    for file in &copied {
        sums.push_str(&format!("{}  {}\n", sha256(file)?, file_name(file)?));
    }
    fs::write(out.join("SHA256SUMS"), sums).map_err(|e| e.to_string())?;
    fs::write(out.join("PREPARED"), b"Prepared locally, not signed or published. Secret/license/source completeness requires release review. Do not equate torrent hashes with release authority.\n").map_err(|e| e.to_string())?;
    fs::remove_file(out.join("PREPARING")).map_err(|e| e.to_string())?;
    println!("Prepared {} (not published or seeded)", out.display());
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<_> = env::args().skip(1).collect();
    if args == ["--help"] {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    }
    match parse(args).and_then(prepare) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("fungos-release: {e}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn portable_names_reject_hidden_paths_and_options() {
        for s in ["../private", ".env", "--secret", "two words", "a/b", "x\n"] {
            assert!(!safe_name(s));
        }
        assert!(safe_name("fungos-cloud-standard-amd64.img.xz"));
    }
    #[test]
    fn digests_are_strict() {
        assert!(digest_valid(&"a".repeat(64)));
        for s in ["A".repeat(64), "f".repeat(63), "g".repeat(64)] {
            assert!(!digest_valid(&s));
        }
    }
    #[test]
    fn webseed_rejects_credentials_and_ambiguous_paths() {
        assert!(webseed_valid(
            "https://downloads.fungos.dev/experimental/2026-10-05/"
        ));
        for s in [
            "http://example.org",
            "https://user:pass@example.org",
            "https:///",
            "https://x/?key=s",
            "https://x/#y",
            "https://x/../private",
            "https://x/\n",
        ] {
            assert!(!webseed_valid(s));
        }
    }
    #[test]
    fn upstream_command_uses_explicit_single_file_without_private_tracker() {
        let args = torrent_args(
            Path::new("/release/image.img.xz"),
            Path::new("/release/image.img.xz.torrent"),
            Some("https://downloads.fungos.dev/r/"),
        )
        .unwrap();
        assert!(args.contains(&"https://downloads.fungos.dev/r/image.img.xz".into()));
        assert_eq!(args.last().unwrap(), "/release/image.img.xz");
        assert!(!args.contains(&"--private".into()));
        assert!(args.contains(&"--anonymize".into()));
    }
    #[test]
    fn malformed_cli_never_touches_output() {
        assert!(parse(vec![]).is_err());
        assert!(parse(vec!["bad".into(); 8]).is_err());
    }

    #[test]
    #[ignore = "requires installed Transmission tools; creates only synthetic fixtures"]
    fn upstream_tools_prepare_image_and_source_torrents() {
        let root = env::temp_dir().join(format!(
            "fungos-release-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(root.clone());
        let names = [
            "fixture.img",
            "sources.tar",
            "licenses.txt",
            "packages.tsv",
            "build.txt",
            "boot.txt",
        ];
        let files: Vec<_> = names
            .iter()
            .map(|name| {
                let p = root.join(name);
                fs::write(&p, b"synthetic test fixture, not an OS image\n").unwrap();
                p
            })
            .collect();
        let expected = sha256(&files[0]).unwrap();
        let out = root.join("release");
        let mut args = vec![files[0].to_str().unwrap().into(), expected.clone()];
        args.extend(files[1..].iter().map(|p| p.to_str().unwrap().into()));
        args.push(out.to_str().unwrap().into());
        let mut wrong = args.clone();
        wrong[1] = "0".repeat(64);
        assert!(prepare(parse(wrong).unwrap()).is_err());
        assert!(!out.exists());
        prepare(parse(args.clone()).unwrap()).unwrap();
        assert!(out.join("PREPARED").is_file());
        assert!(!out.join("PREPARING").exists());
        assert!(out.join("fixture.img.torrent").is_file());
        assert!(out.join("sources.tar.torrent").is_file());
        assert_eq!(sha256(&out.join("fixture.img")).unwrap(), expected);
        assert_eq!(
            fs::read_to_string(out.join("SHA256SUMS"))
                .unwrap()
                .lines()
                .count(),
            8
        );
        assert!(
            parse(args).is_err(),
            "existing releases must not be overwritten"
        );
        let link = root.join("symlink.img");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&files[0], &link).unwrap();
            let mut linked = vec![link.to_str().unwrap().into(), expected];
            linked.extend(files[1..].iter().map(|p| p.to_str().unwrap().into()));
            linked.push(root.join("other").to_str().unwrap().into());
            assert!(parse(linked).is_err());
        }
    }

    #[test]
    #[ignore = "requires Transmission daemon/remote; synthetic loopback-only seed and restart"]
    fn seed_verifies_and_retains_active_state_after_restart() {
        use std::{
            net::TcpListener,
            process::{Child, Stdio},
            time::{Duration, Instant},
        };
        let root = env::temp_dir().join(format!(
            "fungos-seed-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(root.clone());
        let payload = root.join("seed-fixture.img");
        fs::write(&payload, b"synthetic torrent fixture, not fungOS\n").unwrap();
        let torrent = root.join("seed-fixture.img.torrent");
        run(
            "transmission-create",
            &torrent_args(&payload, &torrent, None).unwrap(),
        )
        .unwrap();
        let rpc_reservation = TcpListener::bind("127.0.0.1:0").unwrap();
        let peer_reservation = TcpListener::bind("127.0.0.1:0").unwrap();
        let rpc_port = rpc_reservation.local_addr().unwrap().port();
        let peer_port = peer_reservation.local_addr().unwrap().port();
        let endpoint = format!("127.0.0.1:{rpc_port}");
        let config = root.join("state");
        fs::create_dir(&config).unwrap();
        // 4.1+ snake_case via explicit config format; no public discovery or WAN probe.
        fs::write(
            config.join("settings.json"),
            br#"{
            "pex_enabled": false, "ip_endpoints_ipv4": [], "ip_endpoints_ipv6": [],
            "rpc_host_whitelist": "localhost", "rpc_whitelist": "127.0.0.1",
            "seed_ratio_limited": false, "idle_seeding_limit_enabled": false
        }"#,
        )
        .unwrap();
        struct Daemon(Child);
        impl Drop for Daemon {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let start = || {
            Daemon(
                Command::new("transmission-daemon")
                    .env("TR_SAVE_VERSION_FORMAT", "5")
                    .args([
                        "--foreground",
                        "--no-auth",
                        "--no-dht",
                        "--no-lpd",
                        "--no-portmap",
                        "--bind-address-ipv4",
                        "127.0.0.1",
                        "--bind-address-ipv6",
                        "::1",
                        "--rpc-bind-address",
                        "127.0.0.1",
                        "--allowed",
                        "127.0.0.1",
                        "--port",
                        &rpc_port.to_string(),
                        "--peerport",
                        &peer_port.to_string(),
                        "--config-dir",
                        config.to_str().unwrap(),
                        "--download-dir",
                        root.to_str().unwrap(),
                    ])
                    .stdout(Stdio::null())
                    .stderr(Stdio::from(
                        fs::File::create(root.join("daemon.log")).unwrap(),
                    ))
                    .spawn()
                    .unwrap(),
            )
        };
        let remote = |args: &[&str]| {
            let mut all = vec![endpoint.clone()];
            all.extend(args.iter().map(|s| s.to_string()));
            run("transmission-remote", &all)
        };
        let until = |action: &mut dyn FnMut() -> bool| {
            let deadline = Instant::now() + Duration::from_secs(15);
            while !action() {
                assert!(
                    Instant::now() < deadline,
                    "loopback seed did not reach expected state; RPC {:?}; log {:?}",
                    remote(&["--torrent", "all", "--info"]),
                    fs::read_to_string(root.join("daemon.log"))
                );
                std::thread::sleep(Duration::from_millis(100));
            }
        };
        drop(rpc_reservation);
        drop(peer_reservation);
        let mut daemon = start();
        until(&mut || remote(&["--list"]).is_ok());
        remote(&[
            "--start-paused",
            "--download-dir",
            root.to_str().unwrap(),
            "--add",
            torrent.to_str().unwrap(),
        ])
        .unwrap();
        remote(&["--torrent", "all", "--verify"]).unwrap();
        until(&mut || {
            remote(&["--torrent", "all", "--info"]).is_ok_and(|s| s.contains("Percent Done: 100%"))
        });
        remote(&["--torrent", "all", "--start"]).unwrap();
        until(&mut || {
            remote(&["--torrent", "all", "--info"])
                .is_ok_and(|s| s.contains("State: Seeding") || s.contains("State: Idle"))
        });
        let before = sha256(&payload).unwrap();
        remote(&["--exit"]).unwrap();
        until(&mut || daemon.0.try_wait().unwrap().is_some());
        drop(daemon);
        let mut daemon = start();
        until(&mut || {
            remote(&["--torrent", "all", "--info"]).is_ok_and(|s| {
                (s.contains("State: Seeding") || s.contains("State: Idle"))
                    && s.contains("Percent Done: 100%")
            })
        });
        assert_eq!(sha256(&payload).unwrap(), before);
        remote(&["--exit"]).unwrap();
        until(&mut || daemon.0.try_wait().unwrap().is_some());
    }
}
