//! Actual empty-client transfer; synthetic bytes, loopback-only, never public.
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
fn run(program: &str, args: &[&str]) -> String {
    let out = Command::new(program).args(args).output().unwrap();
    assert!(
        out.status.success(),
        "{program}: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}
struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
struct Daemon(Child);
impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
struct Tracker {
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Drop for Tracker {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap();
        }
    }
}
fn wait(mut condition: impl FnMut() -> bool) {
    let until = Instant::now() + Duration::from_secs(60);
    while !condition() {
        assert!(
            Instant::now() < until,
            "loopback transfer deadline exceeded"
        );
        thread::sleep(Duration::from_millis(100));
    }
}
fn remote(endpoint: &str, args: &[&str]) -> std::process::Output {
    Command::new("transmission-remote")
        .arg(endpoint)
        .args(args)
        .output()
        .unwrap()
}
fn remote_ok(endpoint: &str, args: &[&str]) -> String {
    let out = remote(endpoint, args);
    assert!(
        out.status.success(),
        "RPC {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}
fn daemon(root: &Path, label: &str, rpc: u16, peer: u16, download: &Path) -> Daemon {
    let config = root.join(label);
    fs::create_dir(&config).unwrap();
    fs::write(config.join("settings.json"), br#"{"pex_enabled":false,"ip_endpoints_ipv4":[],"ip_endpoints_ipv6":[],"rpc_host_whitelist":"localhost","rpc_whitelist":"127.0.0.1","seed_ratio_limited":false,"idle_seeding_limit_enabled":false}"#).unwrap();
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
                &rpc.to_string(),
                "--peerport",
                &peer.to_string(),
                "--config-dir",
                config.to_str().unwrap(),
                "--download-dir",
                download.to_str().unwrap(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::from(
                fs::File::create(root.join(format!("{label}.log"))).unwrap(),
            ))
            .spawn()
            .unwrap(),
    )
}
#[test]
#[ignore = "requires Transmission4.1 tools; two loopback clients and bounded test tracker"]
fn empty_client_downloads_matching_bytes_from_loopback_seed() {
    let root = std::env::temp_dir().join(format!(
        "fungos-torrent-download-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let _cleanup = Scratch(root.clone());
    let seed = root.join("seed");
    let download = root.join("download");
    fs::create_dir(&seed).unwrap();
    fs::create_dir(&download).unwrap();
    let payload = seed.join("synthetic-not-fungos.img");
    let bytes: Vec<u8> = (0..8 * 1024 * 1024)
        .map(|n| ((n * 31 + n / 251) % 256) as u8)
        .collect();
    fs::write(&payload, bytes).unwrap();
    let expected = run("sha256sum", &[payload.to_str().unwrap()])
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned();
    let reservations: Vec<_> = (0..4)
        .map(|_| TcpListener::bind("127.0.0.1:0").unwrap())
        .collect();
    let ports: Vec<_> = reservations
        .iter()
        .map(|r| r.local_addr().unwrap().port())
        .collect();
    let tracker_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    tracker_listener.set_nonblocking(true).unwrap();
    let announce = format!(
        "http://127.0.0.1:{}/announce",
        tracker_listener.local_addr().unwrap().port()
    );
    let stop = Arc::new(AtomicBool::new(false));
    let halt = stop.clone();
    let seed_port = ports[1];
    let leecher_port = ports[3];
    let worker = thread::spawn(move || {
        while !halt.load(Ordering::SeqCst) {
            match tracker_listener.accept() {
                Ok((mut stream, address)) => {
                    assert!(address.ip().is_loopback());
                    stream
                        .set_read_timeout(Some(Duration::from_millis(250)))
                        .unwrap();
                    stream
                        .set_write_timeout(Some(Duration::from_millis(250)))
                        .unwrap();
                    let mut request = [0u8; 8192];
                    let _ = stream.read(&mut request);
                    // Discovery only: this tiny test tracker never serves payload
                    // bytes. Both compact peer addresses remain strictly local.
                    let mut body =
                        b"d8:intervali1e8:completei1e10:incompletei1e5:peers12:".to_vec();
                    for port in [seed_port, leecher_port] {
                        body.extend([127, 0, 0, 1]);
                        body.extend(port.to_be_bytes());
                    }
                    body.push(b'e');
                    let header = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
                    let _ = stream.write_all(header.as_bytes());
                    let _ = stream.write_all(&body);
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10))
                }
                Err(error) => panic!("tracker failed: {error}"),
            }
        }
    });
    let _tracker = Tracker {
        stop,
        worker: Some(worker),
    };
    let torrent = root.join("test-only.torrent");
    run(
        "transmission-create",
        &[
            "--anonymize",
            "--piecesize",
            "1024",
            "--tracker",
            &announce,
            "--outfile",
            torrent.to_str().unwrap(),
            payload.to_str().unwrap(),
        ],
    );
    drop(reservations);
    let mut seed_daemon = daemon(&root, "seed-state", ports[0], ports[1], &seed);
    let mut leecher_daemon = daemon(&root, "download-state", ports[2], ports[3], &download);
    let seed_rpc = format!("127.0.0.1:{}", ports[0]);
    let download_rpc = format!("127.0.0.1:{}", ports[2]);
    wait(|| {
        remote(&seed_rpc, &["--list"]).status.success()
            && remote(&download_rpc, &["--list"]).status.success()
    });
    remote_ok(
        &seed_rpc,
        &["--start-paused", "--add", torrent.to_str().unwrap()],
    );
    remote_ok(&seed_rpc, &["--torrent", "all", "--verify"]);
    wait(|| remote_ok(&seed_rpc, &["--torrent", "all", "--info"]).contains("Percent Done: 100%"));
    remote_ok(&seed_rpc, &["--torrent", "all", "--start"]);
    assert!(
        !download.join("synthetic-not-fungos.img").exists(),
        "download client must start empty"
    );
    remote_ok(&download_rpc, &["--add", torrent.to_str().unwrap()]);
    remote_ok(&download_rpc, &["--torrent", "all", "--start"]);
    wait(|| {
        remote_ok(&download_rpc, &["--torrent", "all", "--info"]).contains("Percent Done: 100%")
    });
    let received = run(
        "sha256sum",
        &[download.join("synthetic-not-fungos.img").to_str().unwrap()],
    )
    .split_whitespace()
    .next()
    .unwrap()
    .to_owned();
    assert_eq!(received, expected);
    remote_ok(&seed_rpc, &["--exit"]);
    remote_ok(&download_rpc, &["--exit"]);
    wait(|| {
        seed_daemon.0.try_wait().unwrap().is_some()
            && leecher_daemon.0.try_wait().unwrap().is_some()
    });
    println!("FUNGOS_LOOPBACK_TWO_CLIENT_TRANSFER_VERIFIED sha256={received} bytes=8388608 (synthetic fixture, no public discovery)");
}
