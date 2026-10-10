//! Private guest-only proof of the real signed APT engine, never a fleet tool.
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const QA: &str = "/opt/fungos-mycelium-apt-qa";
const BIN: &str = "/usr/bin/mycelium";
const STATE: &str = "/var/lib/fungos-mycelium-apt-proof";
const CHECKPOINT: &str = "/var/lib/mycelium/apt/activation-pending.json";
const POLICY: &str = "/var/lib/mycelium/software-services.json";
fn run(program: &str, args: &[&str]) -> Result<String> {
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
    Ok(String::from_utf8(out.stdout)?)
}
fn require(value: bool, error: &str) -> Result<()> {
    if value {
        Ok(())
    } else {
        Err(error.into())
    }
}
fn sha(path: &str) -> Result<String> {
    Ok(run("sha256sum", &[path])?
        .split_whitespace()
        .next()
        .ok_or("missing digest")?
        .to_owned())
}
fn version() -> Result<String> {
    Ok(run("dpkg-query", &["-W", "-f=${Version}", "mycelium"])?
        .trim()
        .to_owned())
}
fn private(path: &str, bytes: &[u8]) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::write(path, bytes)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}
fn identity() -> Result<String> {
    let mut result = String::new();
    for name in [
        "peer.key",
        "iroh-secret.key",
        "pki/node-key.pem",
        "pki/node.pem",
        "node.env",
    ] {
        result.push_str(&format!(
            "{} {name}\n",
            sha(&format!("/var/lib/mycelium/{name}"))?
        ));
    }
    result.push_str(&format!(
        "{} ssh-host-public\n",
        sha("/etc/ssh/ssh_host_ed25519_key.pub")?
    ));
    Ok(result)
}
fn healthy() -> Result<()> {
    run(BIN, &["software", "apt", "health", "mycelium", BIN])?;
    for file in [BIN, "/usr/lib/systemd/system/mycelium.service"] {
        require(
            run("dpkg-query", &["-S", file])?.trim() == format!("mycelium: {file}"),
            "package owner changed",
        )?;
    }
    for file in [
        "/var/lib/mycelium/bin/mycelium",
        "/etc/systemd/system/mycelium.service",
    ] {
        require(
            fs::symlink_metadata(file).is_err(),
            "native shadow appeared",
        )?;
    }
    Ok(())
}
fn wait_health() -> Result<()> {
    let until = Instant::now() + Duration::from_secs(40);
    loop {
        if healthy().is_ok() {
            return Ok(());
        }
        require(Instant::now() < until, "service health timeout")?;
        thread::sleep(Duration::from_millis(100));
    }
}
fn plan(candidate: &str) -> Result<(String, String)> {
    let output = run(
        BIN,
        &["software", "apt", "plan", "mycelium", candidate, BIN],
    )?;
    let digest = plan_digest(&output)?;
    let path = format!("{QA}/{candidate}.plan.json");
    private(&path, output.as_bytes())?;
    Ok((path, digest))
}
fn plan_digest(output: &str) -> Result<String> {
    let digest = output
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("\"digest\": \"")
                .and_then(|s| s.strip_suffix("\","))
        })
        .ok_or("missing plan digest")?
        .to_owned();
    require(
        digest.len() == 64 && digest.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid plan digest",
    )?;
    Ok(digest)
}
fn apply(path: &str, digest: &str) -> Result<String> {
    run(BIN, &["software", "apt", "apply", path, digest, "--write"])
}
fn rejected(result: Result<String>, expected: &str) -> Result<()> {
    match result {
        Err(error) if error.to_string().contains(expected) => Ok(()),
        other => Err(format!("expected rejection {expected}: {other:?}").into()),
    }
}
/// Transparent QA fault injection: kill only the supervised process executing
/// the exact candidate bytes. The normal service-owned readiness check remains
/// unchanged. This tests recovery mechanics, not an intrinsically broken build.
struct Fault {
    stop: Arc<AtomicBool>,
    hits: Arc<AtomicUsize>,
    worker: Option<thread::JoinHandle<()>>,
}
impl Fault {
    fn candidate(digest: String) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let hits = Arc::new(AtomicUsize::new(0));
        let halt = stop.clone();
        let count = hits.clone();
        let worker = thread::spawn(move || {
            while !halt.load(Ordering::SeqCst) {
                if let Ok(pid) = run(
                    "systemctl",
                    &["show", "mycelium.service", "--property=MainPID", "--value"],
                ) {
                    let pid = pid.trim();
                    if pid.parse::<u32>().is_ok_and(|v| v > 1)
                        && sha(&format!("/proc/{pid}/exe")).is_ok_and(|v| v == digest)
                    {
                        if run("kill", &["-KILL", pid]).is_ok() {
                            count.fetch_add(1, Ordering::SeqCst);
                        }
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
        });
        Self {
            stop,
            hits,
            worker: Some(worker),
        }
    }
}
impl Drop for Fault {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
fn main() -> Result<()> {
    require(run("id", &["-u"])?.trim() == "0", "root required")?;
    require(
        fs::read_to_string("/sys/class/dmi/id/product_name")?.trim()
            == "fungos-mycelium-apt-validation",
        "refusing nonqualification guest",
    )?;
    require(
        Path::new("/var/lib/fungos-mycelium-package-proof").is_file(),
        "missing prerequisite genuine package proof",
    )?;
    std::env::set_var("MYCELIUM_HOME", "/var/lib/mycelium");
    let candidate = "0.1.0+signedqa2";
    let failed = "0.1.0+signedqa3";
    if Path::new(STATE).is_file() {
        require(
            identity()? == fs::read_to_string(STATE)?,
            "identity changed across reboot",
        )?;
        require(
            version()? == candidate,
            "healthy candidate not retained across reboot",
        )?;
        wait_health()?;
        require(
            !Path::new(CHECKPOINT).exists(),
            "pending recovery after reboot",
        )?;
        println!("FUNGOS_MYCELIUM_SIGNED_APT_REBOOT_VERIFIED");
        run("systemctl", &["--no-block", "poweroff"])?;
        return Ok(());
    }
    wait_health()?;
    let before = identity()?;
    let baseline_sha = sha(BIN)?;
    require(version()? == "0.1.0", "unexpected baseline version")?;
    run("apt-get", &["update", "--error-on=any"])?;
    let (path, digest) = plan(candidate)?;
    rejected(apply(&path, &"0".repeat(64)), "APT plan digest mismatch")?;
    require(
        version()? == "0.1.0" && sha(BIN)? == baseline_sha,
        "digest rejection mutated executable",
    )?;
    println!("FUNGOS_MYCELIUM_APT_EXACT_DIGEST_REJECTION_VERIFIED");
    let key = format!("{QA}/repository/fungos-archive-keyring.gpg");
    let key_bytes = fs::read(&key)?;
    fs::remove_file(&key)?;
    let rejection = apply(&path, &digest);
    fs::write(&key, key_bytes)?;
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&key, fs::Permissions::from_mode(0o644))?;
    }
    rejected(rejection, "apt-get failed")?;
    require(version()? == "0.1.0", "missing key changed version")?;
    println!("FUNGOS_MYCELIUM_APT_MISSING_SIGNED_BY_KEY_REJECTED");
    let signature = format!("{QA}/repository/dists/testing/InRelease");
    let signed = fs::read(&signature)?;
    let altered = String::from_utf8(signed.clone())?.replace("Origin: fungOS", "Origin: tampered");
    require(
        altered.as_bytes() != signed,
        "signature test did not alter signed bytes",
    )?;
    fs::write(&signature, altered)?;
    let rejection = apply(&path, &digest);
    fs::write(&signature, signed)?;
    rejected(rejection, "apt-get failed")?;
    require(version()? == "0.1.0", "invalid signature changed version")?;
    println!("FUNGOS_MYCELIUM_APT_BAD_SIGNATURE_REJECTED");
    let policy = fs::read(POLICY)?;
    let changed =
        String::from_utf8(policy.clone())?.replace("\"timeout_secs\":30", "\"timeout_secs\":29");
    require(
        changed.as_bytes() != policy,
        "stale plan test did not alter binding",
    )?;
    private(POLICY, changed.as_bytes())?;
    let rejection = apply(&path, &digest);
    private(POLICY, &policy)?;
    rejected(rejection, "APT plan is stale")?;
    println!("FUNGOS_MYCELIUM_APT_STALE_PLAN_REJECTED");
    apply(&path, &digest)?;
    wait_health()?;
    let candidate_sha = sha(BIN)?;
    require(
        candidate_sha != baseline_sha,
        "same executable is not changed-binary proof",
    )?;
    require(
        run(BIN, &["_self-check"])?.contains("iroh-sync=true"),
        "new capability diagnostic missing",
    )?;
    require(
        version()? == candidate && identity()? == before,
        "candidate identity or version mismatch",
    )?;
    println!("FUNGOS_MYCELIUM_SIGNED_APT_CHANGED_BINARY_VERIFIED");
    let (bad_path, bad_digest) = plan(failed)?;
    let fault = Fault::candidate(baseline_sha.clone());
    let result = apply(&bad_path, &bad_digest);
    require(
        fault.hits.load(Ordering::SeqCst) > 0,
        "fault injection never reached candidate process",
    )?;
    drop(fault);
    rejected(result, "previous package restored and health verified")?;
    wait_health()?;
    require(
        version()? == candidate && sha(BIN)? == candidate_sha && identity()? == before,
        "rollback did not restore healthy identity",
    )?;
    require(
        !Path::new(CHECKPOINT).exists(),
        "rollback left pending checkpoint",
    )?;
    println!("FUNGOS_MYCELIUM_SIGNED_APT_FAILED_HEALTH_ROLLBACK_VERIFIED");
    let (interrupted_path, interrupted_digest) = plan(failed)?;
    let fault = Fault::candidate(baseline_sha);
    let log = fs::File::create(format!("{QA}/interrupted.log"))?;
    let mut child = Command::new(BIN)
        .args([
            "software",
            "apt",
            "apply",
            &interrupted_path,
            &interrupted_digest,
            "--write",
        ])
        .stdout(Stdio::from(log.try_clone()?))
        .stderr(Stdio::from(log))
        .spawn()?;
    let until = Instant::now() + Duration::from_secs(30);
    while !Path::new(CHECKPOINT).exists() || fault.hits.load(Ordering::SeqCst) == 0 {
        require(
            child.try_wait()?.is_none(),
            "apply exited before interruption gate",
        )?;
        require(Instant::now() < until, "interruption checkpoint timeout")?;
        thread::sleep(Duration::from_millis(20));
    }
    child.kill()?;
    child.wait()?;
    drop(fault);
    require(
        Path::new(CHECKPOINT).exists(),
        "interrupt did not retain genuine checkpoint",
    )?;
    run(BIN, &["software", "apt", "recover", "--write"])?;
    wait_health()?;
    require(
        version()? == candidate && sha(BIN)? == candidate_sha && identity()? == before,
        "recover did not restore identity and executable",
    )?;
    require(
        !Path::new(CHECKPOINT).exists(),
        "recover did not clear checkpoint",
    )?;
    println!("FUNGOS_MYCELIUM_SIGNED_APT_INTERRUPTED_RECOVER_VERIFIED");
    private(STATE, before.as_bytes())?;
    run("sync", &[])?;
    run("systemctl", &["--no-block", "reboot"])?;
    Ok(())
}
#[cfg(test)]
mod tests {
    #[test]
    fn plan_digest_requires_exact_hex() {
        let expected = "a".repeat(64);
        assert_eq!(
            super::plan_digest(&format!(
                "{{\n  \"digest\": \"{expected}\",\n  \"plan\": {{}}\n}}"
            ))
            .unwrap(),
            expected
        );
        assert!(super::plan_digest("{}").is_err());
        assert!(super::plan_digest("\"digest\": \"bad\",").is_err());
    }
}
