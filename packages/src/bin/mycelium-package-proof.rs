//! Private disposable guest qualification of genuine Mycelium Debian packages.
use std::{
    fs,
    io::{BufRead, BufReader},
    net::{SocketAddr, TcpStream},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const QA: &str = "/opt/fungos-mycelium-package-qa";
const STATE: &str = "/var/lib/fungos-mycelium-package-proof";
const BINARY: &str = "/usr/bin/mycelium";
const UNIT: &str = "/usr/lib/systemd/system/mycelium.service";

fn command(program: &str, args: &[&str]) -> Result<String> {
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
fn digest(path: &str) -> Result<String> {
    command("sha256sum", &[path])?
        .split_whitespace()
        .next()
        .map(str::to_owned)
        .ok_or_else(|| "missing digest".into())
}
fn private(path: &str, bytes: &[u8]) -> Result<()> {
    fs::write(path, bytes)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}
fn require(condition: bool, message: &str) -> Result<()> {
    if !condition {
        return Err(message.to_owned().into());
    }
    Ok(())
}
fn no_shadow() -> Result<()> {
    for path in [
        "/var/lib/mycelium/bin/mycelium",
        "/root/.mycelium/bin/mycelium",
        "/etc/systemd/system/mycelium.service",
        "/run/systemd/system/mycelium.service",
        "/root/.config/systemd/user/mycelium.service",
    ] {
        require(
            fs::symlink_metadata(path).is_err(),
            "native binary or unit shadow appeared",
        )?;
    }
    Ok(())
}
fn owned() -> Result<String> {
    for path in [BINARY, UNIT] {
        require(
            command("dpkg-query", &["-S", path])?.trim() == format!("mycelium: {path}"),
            "package ownership mismatch",
        )?;
    }
    no_shadow()?;
    Ok(format!(
        "{} {}\n{} {}\n",
        digest(BINARY)?,
        BINARY,
        digest(UNIT)?,
        UNIT
    ))
}
fn identity() -> Result<String> {
    let mut value = String::new();
    for name in [
        "peer.key",
        "iroh-secret.key",
        "pki/node-key.pem",
        "pki/node.pem",
        "node.env",
    ] {
        let path = format!("/var/lib/mycelium/{name}");
        value.push_str(&format!("{} {name}\n", digest(&path)?));
    }
    value.push_str(&format!(
        "{} ssh-host-public\n",
        digest("/etc/ssh/ssh_host_ed25519_key.pub")?
    ));
    Ok(value)
}
fn active() -> Result<()> {
    require(
        command("systemctl", &["is-active", "mycelium.service"])?.trim() == "active",
        "daemon inactive",
    )?;
    let until = Instant::now() + Duration::from_secs(30);
    while command(BINARY, &["daemon", "status"]).is_err() {
        require(Instant::now() < until, "daemon did not become healthy")?;
        thread::sleep(Duration::from_millis(100));
    }
    let pid = command(
        "systemctl",
        &["show", "mycelium.service", "--property=MainPID", "--value"],
    )?;
    require(pid.trim().parse::<u32>()? > 1, "daemon has no process")?;
    require(
        fs::read_link(format!("/proc/{}/exe", pid.trim()))? == Path::new(BINARY),
        "daemon uses nonpackage executable",
    )?;
    require(
        digest(&format!("/proc/{}/exe", pid.trim()))? == digest(BINARY)?,
        "running executable digest mismatch",
    )?;
    Ok(())
}
fn native_rejection(args: &[&str]) -> Result<()> {
    let before = owned()?;
    let out = Command::new(BINARY).args(args).output()?;
    require(!out.status.success(), "native activation accepted")?;
    require(
        String::from_utf8(out.stderr)?.contains("owned by Debian package mycelium"),
        "not an ownership rejection",
    )?;
    require(
        owned()? == before,
        "ownership refusal modified package files",
    )?;
    for path in [
        "/usr/bin/mycelium.previous",
        "/usr/bin/mycelium.update",
        "/var/lib/mycelium/update-state.json",
    ] {
        require(
            !Path::new(path).exists(),
            "native activation wrote state before refusal",
        )?;
    }
    Ok(())
}
struct Authority(Child);
impl Drop for Authority {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn enroll() -> Result<()> {
    fs::create_dir_all("/run/fungos-mycelium-package-qa")?;
    fs::set_permissions(
        "/run/fungos-mycelium-package-qa",
        fs::Permissions::from_mode(0o700),
    )?;
    let authority = "/run/fungos-mycelium-package-qa/authority";
    command(BINARY, &["enroll", "init", "--path", authority, "--write"])?;
    let mut pair = Authority(
        Command::new(BINARY)
            .args([
                "pair",
                "--kind",
                "peer",
                "--name",
                "package-qa",
                "--site",
                "qualification",
                "--peer",
                "127.0.0.1:65534",
                "--enrollment-ca",
                authority,
                "--listen",
                "127.0.0.1:18788",
                "--advertise",
                "http://127.0.0.1:18788",
                "--invite-store",
                "/run/fungos-mycelium-package-qa/invites.json",
                "--ttl",
                "15m",
            ])
            .env(
                "MYCELIUM_HOME",
                "/run/fungos-mycelium-package-qa/authority-state",
            )
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?,
    );
    let mut lines = BufReader::new(pair.0.stdout.take().ok_or("pair stdout missing")?);
    let claim = loop {
        let mut line = String::new();
        require(
            lines.read_line(&mut line)? != 0,
            "pair failed before generating claim",
        )?;
        if let Some(value) = line.strip_prefix("Pairing claim: ") {
            break value.trim().to_owned();
        }
    };
    fs::create_dir_all("/run/fungos-first-contact")?;
    fs::set_permissions(
        "/run/fungos-first-contact",
        fs::Permissions::from_mode(0o700),
    )?;
    private("/run/fungos-first-contact/claim", claim.as_bytes())?;
    fs::create_dir_all("/etc/fungos")?;
    private(
        "/etc/fungos/first-contact.env",
        b"PROFILE=fungos-headless-edge\n",
    )?;
    let until = Instant::now() + Duration::from_secs(10);
    let address: SocketAddr = "127.0.0.1:18788".parse()?;
    while TcpStream::connect_timeout(&address, Duration::from_millis(100)).is_err() {
        require(Instant::now() < until, "pair listener did not become ready")?;
        thread::sleep(Duration::from_millis(100));
    }
    require(
        command(
            "systemctl",
            &[
                "show",
                "fungos-first-contact.service",
                "--property=ProtectSystem",
                "--value",
            ],
        )?
        .trim()
            == "strict",
        "first-contact sandbox not strict",
    )?;
    require(
        command(
            "systemctl",
            &[
                "show",
                "fungos-first-contact.service",
                "--property=ProtectHome",
                "--value",
            ],
        )?
        .trim()
            == "yes",
        "first-contact home not protected",
    )?;
    command(
        "systemctl",
        &["reset-failed", "fungos-first-contact.service"],
    )?;
    command("systemctl", &["start", "fungos-first-contact.service"])?;
    require(
        !Path::new("/run/fungos-first-contact/claim").exists(),
        "successful claim not consumed",
    )?;
    require(
        command(
            "systemctl",
            &[
                "show",
                "fungos-first-contact.service",
                "--property=Result",
                "--value",
            ],
        )?
        .trim()
            == "success",
        "first contact failed",
    )?;
    active()?;
    println!("FUNGOS_MYCELIUM_FIRST_CONTACT_SANDBOX_VERIFIED");
    // Exercise Iroh key persistence independently of HTTP enrollment; no relay/NIC.
    command(BINARY, &["sync", "enable", "--write"])?;
    let mut env = fs::read_to_string("/var/lib/mycelium/node.env")?;
    env.push_str("MYCELIUM_IROH_SERVER_NAME=package-qa\n");
    private("/var/lib/mycelium/node.env", env.as_bytes())?;
    command("systemctl", &["restart", "mycelium.service"])?;
    let until = Instant::now() + Duration::from_secs(30);
    while !Path::new("/var/lib/mycelium/iroh-secret.key").exists() {
        require(Instant::now() < until, "Iroh key missing")?;
        thread::sleep(Duration::from_millis(100));
    }
    active()
}
fn installed_version() -> Result<String> {
    Ok(command("dpkg-query", &["-W", "-f=${Version}", "mycelium"])?
        .trim()
        .to_owned())
}
fn main() -> Result<()> {
    require(command("id", &["-u"])?.trim() == "0", "root required")?;
    require(
        fs::read_to_string("/sys/class/dmi/id/product_name")?.trim()
            == "fungos-mycelium-package-validation",
        "refusing nonqualification guest",
    )?;
    std::env::set_var("MYCELIUM_HOME", "/var/lib/mycelium");
    if Path::new(STATE).is_file() {
        require(
            identity()? == fs::read_to_string(STATE)?,
            "identity/configuration changed over reboot",
        )?;
        active()?;
        owned()?;
        require(
            !command(
                "systemctl",
                &[
                    "show",
                    "fungos-first-contact.service",
                    "--property=ActiveState",
                    "--value",
                ],
            )?
            .contains("failed"),
            "first contact retried without a runtime claim",
        )?;
        println!("FUNGOS_MYCELIUM_PACKAGE_REBOOT_VERIFIED");
        return command("systemctl", &["poweroff"]).map(|_| ());
    }
    require(
        !Path::new("/var/lib/mycelium").exists(),
        "guest contains an existing identity",
    )?;
    let a = format!("{QA}/a.deb");
    let b = format!("{QA}/b.deb");
    for (path, pin) in [(&a, "a.sha256"), (&b, "b.sha256")] {
        require(
            digest(path)? == fs::read_to_string(format!("{QA}/{pin}"))?.trim(),
            "package digest mismatch",
        )?;
        require(
            command("dpkg-deb", &["-f", path, "Package"])?.trim() == "mycelium",
            "not owner package",
        )?;
    }
    let first_version = command("dpkg-deb", &["-f", &a, "Version"])?;
    let next_version = command("dpkg-deb", &["-f", &b, "Version"])?;
    require(
        command(
            "dpkg",
            &[
                "--compare-versions",
                first_version.trim(),
                "lt",
                next_version.trim(),
            ],
        )
        .is_ok(),
        "upgrade version is not newer",
    )?;
    command("apt-get", &["-y", "install", &a])?;
    command("systemctl", &["daemon-reload"])?;
    let initial = owned()?;
    command(BINARY, &["_self-check"])?;
    require(
        !Path::new("/var/lib/mycelium/pki").exists(),
        "install generated identity",
    )?;
    require(
        command(
            "systemctl",
            &[
                "show",
                "mycelium.service",
                "--property=ActiveState",
                "--value",
            ],
        )?
        .trim()
            == "inactive",
        "install auto-started",
    )?;
    println!("FUNGOS_MYCELIUM_PACKAGE_INERT_INSTALL_VERIFIED");
    native_rejection(&["update", "apply", "--path", BINARY, "--write"])?;
    std::os::unix::fs::symlink(BINARY, "/run/fungos-mycelium-package-alias")?;
    native_rejection(&[
        "update",
        "apply",
        "--path",
        "/run/fungos-mycelium-package-alias",
        "--write",
    ])?;
    println!("FUNGOS_MYCELIUM_NATIVE_UPDATE_PREWRITE_REJECTION_VERIFIED");
    enroll()?;
    require(owned()? == initial, "claim modified owned files")?;
    let material = identity()?;
    command(BINARY, &["setup", "--packaged", "--repair"])?;
    require(
        identity()? == material && owned()? == initial,
        "repair changed identity or owned files",
    )?;
    println!("FUNGOS_MYCELIUM_PACKAGE_REPAIR_VERIFIED");
    command("apt-get", &["-y", "install", &b])?;
    command("systemctl", &["daemon-reload"])?;
    command("systemctl", &["restart", "mycelium.service"])?;
    active()?;
    require(
        installed_version()? == next_version.trim(),
        "upgrade version mismatch",
    )?;
    require(identity()? == material, "upgrade changed private state")?;
    owned()?;
    println!("FUNGOS_MYCELIUM_REAL_DEB_UPGRADE_VERIFIED");
    command(
        "apt-get",
        &["-y", "--allow-downgrades", "install", &a],
    )?;
    command("systemctl", &["daemon-reload"])?;
    command("systemctl", &["restart", "mycelium.service"])?;
    active()?;
    require(
        installed_version()? == first_version.trim(),
        "downgrade version mismatch",
    )?;
    require(
        identity()? == material && owned()? == initial,
        "downgrade changed state or owned files",
    )?;
    println!("FUNGOS_MYCELIUM_REAL_DEB_DOWNGRADE_VERIFIED");
    private(STATE, material.as_bytes())?;
    println!("FUNGOS_MYCELIUM_PACKAGE_AWAITING_REBOOT");
    command("systemctl", &["poweroff"]).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refusal_is_fail_closed() {
        assert!(require(false, "unsafe").is_err());
    }
    #[test]
    fn guest_marker_is_distinct() {
        assert_ne!(STATE, "/var/lib/mycelium");
        assert!(QA.starts_with("/opt/"));
    }
}
