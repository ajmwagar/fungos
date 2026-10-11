//! Private certificate-only access qualification; never a fleet operation.
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Output},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const CLI: &str = "/usr/bin/mycelium";
const QA: &str = "/var/lib/fungos-access-proof";
fn require(value: bool, why: &str) -> Result<()> {
    if value {
        Ok(())
    } else {
        Err(why.into())
    }
}
fn output(program: &str, args: &[&str]) -> Result<Output> {
    Ok(Command::new(program).args(args).output()?)
}
fn run(program: &str, args: &[&str]) -> Result<String> {
    let out = output(program, args)?;
    require(
        out.status.success(),
        &format!("{program} failed: {}", String::from_utf8_lossy(&out.stderr)),
    )?;
    Ok(String::from_utf8(out.stdout)?)
}
fn private(path: &str, bytes: &[u8]) -> Result<()> {
    fs::write(path, bytes)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}
fn p(name: &str) -> String {
    format!("{QA}/{name}")
}
fn key(name: &str) -> Result<()> {
    run(
        "ssh-keygen",
        &["-q", "-t", "ed25519", "-N", "", "-f", &p(name)],
    )?;
    Ok(())
}
fn healthy() -> Result<()> {
    let until = Instant::now() + Duration::from_secs(30);
    // `daemon status` deliberately exits successfully for "not running".
    // Require a real daemon request, not that informational command's status.
    while !output(CLI, &["access", "list", "--json"])?.status.success() {
        require(Instant::now() < until, "native daemon unhealthy")?;
        thread::sleep(Duration::from_millis(100));
    }
    Ok(())
}
fn grant(name: &str, role: &str, user: &str, serial: u64, now: u64) -> Result<()> {
    let public = fs::read_to_string(p(&format!("{name}.pub")))?;
    require(!public.contains(['"', '\\']), "unsafe public key JSON")?;
    let value = format!(
        r#"{{"kind":"grant","grant_id":"qa-{name}","principal":"qa:{name}","serial":{serial},"roles":["{role}"],"scopes":["site:qualification"],"unix_users":["{user}"],"ssh_public_keys":["{}"],"oidc_audiences":[],"oidc_ssh_key_exchange":false,"not_before":{},"not_after":{}}}"#,
        public.trim(),
        now - 5,
        now + 1800
    );
    private(&p(&format!("{name}.json")), value.as_bytes())?;
    run(
        CLI,
        &[
            "access",
            "publish",
            "--statement",
            &p(&format!("{name}.json")),
            "--signing-key",
            &p("access.key"),
            "--write",
        ],
    )?;
    run(
        CLI,
        &[
            "access",
            "ssh",
            "issue",
            "--grant",
            &format!("qa-{name}"),
            "--public-key",
            &p(&format!("{name}.pub")),
            "--ca",
            &p("user_ca"),
            "--path",
            &p(&format!("{name}-cert.pub")),
            "--ttl",
            "15m",
            "--write",
        ],
    )?;
    Ok(())
}
fn ssh(name: &str, certificate: Option<&str>, command: &str) -> Result<Output> {
    let mut args = vec![
        "-vvv".to_owned(),
        "-F".to_owned(),
        "/dev/null".into(),
        "-o".into(),
        "BatchMode=yes".into(),
        "-o".into(),
        "IdentitiesOnly=yes".into(),
        "-o".into(),
        "IdentityAgent=none".into(),
        "-o".into(),
        "StrictHostKeyChecking=yes".into(),
        "-o".into(),
        format!("UserKnownHostsFile={}", p("known_hosts")),
        "-o".into(),
        "PasswordAuthentication=no".into(),
        "-o".into(),
        "KbdInteractiveAuthentication=no".into(),
        "-o".into(),
        "PreferredAuthentications=publickey".into(),
        "-o".into(),
        "ConnectTimeout=5".into(),
        "-i".into(),
        p(name),
    ];
    if let Some(cert) = certificate {
        args.extend(["-o".into(), format!("CertificateFile={}", p(cert))]);
    }
    args.extend(["qaallowed@127.0.0.1".into(), command.into()]);
    Ok(Command::new("ssh").args(args).output()?)
}
fn denied(out: Output) -> Result<()> {
    require(
        !out.status.success()
            && String::from_utf8_lossy(&out.stderr).contains("Permission denied (publickey)"),
        "not a certificate authentication denial",
    )
}
fn proof() -> Result<()> {
    require(run("id", &["-u"])?.trim() == "0", "root required")?;
    require(
        fs::read_to_string("/sys/class/dmi/id/product_name")?.trim()
            == "fungos-mycelium-access-validation",
        "wrong guest",
    )?;
    require(
        Path::new("/var/lib/fungos-mycelium-package-proof").is_file(),
        "missing prior private package proof",
    )?;
    require(
        !Path::new(QA).exists(),
        "refusing reused access proof state",
    )?;
    std::env::set_var("MYCELIUM_HOME", "/var/lib/mycelium");
    let candidate = "/opt/fungos-access-proof/candidate.deb";
    if Path::new(candidate).is_file() {
        let sha = run("sha256sum", &[candidate])?;
        require(
            sha.split_whitespace().next()
                == Some(fs::read_to_string("/opt/fungos-access-proof/candidate.sha256")?.trim()),
            "candidate package digest mismatch",
        )?;
        run("apt-get", &["-y", "install", candidate])?;
        run("systemctl", &["daemon-reload"])?;
        run("systemctl", &["restart", "mycelium.service"])?;
        let binary_sha = run("sha256sum", &[CLI])?;
        require(
            binary_sha.split_whitespace().next()
                == Some(
                    fs::read_to_string("/opt/fungos-access-proof/candidate-binary.sha256")?.trim(),
                ),
            "candidate executable does not match package payload",
        )?;
        require(
            run("dpkg-query", &["-S", CLI])?.trim() == "mycelium: /usr/bin/mycelium",
            "candidate not package owned",
        )?;
        println!("FUNGOS_ACCESS_GENUINE_CANDIDATE_INSTALLED");
    }
    healthy()?;
    let state = run(CLI, &["access", "list", "--json"])?;
    require(
        state
            .split_whitespace()
            .collect::<String>()
            .contains("\"grants\":[]"),
        "peer already has personal grants",
    )?;
    require(
        !output("getent", &["passwd", "qaallowed"])?.status.success(),
        "peer enrollment created personal account",
    )?;
    fs::create_dir(QA)?;
    fs::set_permissions(QA, fs::Permissions::from_mode(0o700))?;
    for name in ["allowed", "wrongrole", "ungranted"] {
        key(name)?;
    }
    let host = fs::read_to_string("/etc/ssh/ssh_host_ed25519_key.pub")?;
    private(
        &p("known_hosts"),
        format!("127.0.0.1 {}\n", host.trim()).as_bytes(),
    )?;
    denied(ssh("ungranted", None, "id -un")?)?;
    println!("FUNGOS_PEER_ENROLLMENT_NO_PERSONAL_ACCESS_VERIFIED");
    let generated = run(
        CLI,
        &[
            "access",
            "keygen",
            "--path",
            &p("access.key"),
            "--write",
            "--json",
        ],
    )?;
    let signer = generated
        .split("\"signer\"")
        .nth(1)
        .and_then(|v| v.split('"').nth(1))
        .ok_or("signer missing")?;
    require(
        signer.len() == 64 && signer.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid signer ID",
    )?;
    let mut env = fs::read_to_string("/var/lib/mycelium/node.env")?;
    env.push_str(&format!("MYCELIUM_ACCESS_KEYS={signer}\n"));
    private("/var/lib/mycelium/node.env", env.as_bytes())?;
    run("systemctl", &["restart", "mycelium.service"])?;
    healthy()?;
    run(
        CLI,
        &[
            "access",
            "ssh",
            "ca-init",
            "--path",
            &p("user_ca"),
            "--write",
        ],
    )?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    grant("allowed", "qa-operator", "qaallowed", 101, now)?;
    grant("wrongrole", "qa-other", "qawrong", 102, now)?;
    run(
        CLI,
        &[
            "access",
            "ssh",
            "host-policy",
            "set",
            "--role",
            "qa-operator",
            "--ca-public",
            &p("user_ca.pub"),
            "--write",
        ],
    )?;
    run(
        CLI,
        &["access", "ssh", "host-policy", "reconcile", "--dry-run"],
    )?;
    run(
        CLI,
        &["access", "ssh", "host-policy", "reconcile", "--write"],
    )?;
    run(
        CLI,
        &[
            "access",
            "ssh",
            "client-config",
            "--host",
            "qa-host",
            "--hostname",
            "127.0.0.1",
            "--user",
            "qaallowed",
            "--identity",
            &p("allowed"),
            "--certificate",
            &p("allowed-cert.pub"),
            "--path",
            &p("client-config"),
            "--write",
        ],
    )?;
    require(
        fs::metadata(p("client-config"))?.permissions().mode() & 0o777 == 0o600,
        "client config not private",
    )?;
    // Repair mode-only drift left by older enrollment under UMask=0077.
    for directory in ["/etc/ssh/mycelium", "/etc/ssh/mycelium/principals"] {
        fs::set_permissions(directory, fs::Permissions::from_mode(0o700))?;
    }
    run(
        CLI,
        &["access", "ssh", "host-policy", "reconcile", "--write"],
    )?;
    let allowed = ssh("allowed", Some("allowed-cert.pub"), "id -un")?;
    require(
        allowed.status.success() && String::from_utf8(allowed.stdout)?.trim() == "qaallowed",
        &format!(
            "mapped certificate SSH failed: {}",
            String::from_utf8_lossy(&allowed.stderr)
        ),
    )?;
    for directory in ["/etc/ssh/mycelium", "/etc/ssh/mycelium/principals"] {
        require(
            fs::metadata(directory)?.permissions().mode() & 0o777 == 0o755,
            "public SSH policy not traversable under private unit umask",
        )?;
    }
    let groups = run("id", &["-nG", "qaallowed"])?;
    let accepted = run(
        "journalctl",
        &["-b", "0", "--no-pager", "-u", "ssh.service"],
    )?;
    require(
        accepted.lines().any(|line| {
            line.contains("Accepted publickey for qaallowed") && line.contains("ED25519-CERT")
        }),
        "server did not record certificate authentication",
    )?;
    // OpenSSH needs the underlying private-key algorithm enabled to sign a
    // certificate. Prove raw-key fallback cannot succeed using the exact same
    // private key under a basename with no automatically discovered cert.
    private(&p("allowed-raw"), &fs::read(p("allowed"))?)?;
    denied(ssh("allowed-raw", None, "id -un")?)?;
    require(
        !groups
            .split_whitespace()
            .any(|v| ["sudo", "wheel", "admin", "root"].contains(&v)),
        "SSH grant granted privileged group",
    )?;
    let sudo = ssh("allowed", Some("allowed-cert.pub"), "sudo -n true")?;
    require(!sudo.status.success(), "SSH grant granted sudo")?;
    println!("FUNGOS_PERSONAL_CERTIFICATE_ONLY_SSH_VERIFIED");
    denied(ssh("wrongrole", Some("wrongrole-cert.pub"), "id -un")?)?;
    run(
        "ssh-keygen",
        &[
            "-q",
            "-s",
            &p("user_ca"),
            "-I",
            "qa-ungranted",
            "-n",
            "mycelium-role-qa-ungranted",
            "-V",
            "+0s:+15m",
            "-z",
            "103",
            &p("ungranted.pub"),
        ],
    )?;
    denied(ssh("ungranted", Some("ungranted-cert.pub"), "id -un")?)?;
    println!("FUNGOS_SAME_CA_UNGRANTED_AND_WRONG_ROLE_DENIED_VERIFIED");
    let revoke = format!(
        r#"{{"kind":"revoke","revocation_id":"qa-allowed-revoke","grant_id":"qa-allowed","principal":null,"serial":null,"revoked_at":{now},"reason":"isolated access qualification"}}"#
    );
    private(&p("revoke.json"), revoke.as_bytes())?;
    run(
        CLI,
        &[
            "access",
            "publish",
            "--statement",
            &p("revoke.json"),
            "--signing-key",
            &p("access.key"),
            "--write",
        ],
    )?;
    run(
        CLI,
        &["access", "ssh", "host-policy", "reconcile", "--write"],
    )?;
    denied(ssh("allowed", Some("allowed-cert.pub"), "id -un")?)?;
    println!("FUNGOS_REVOKED_CERTIFICATE_DENIED_VERIFIED");
    private(&p("passed"), b"certificate-only allow; wrong-role and same-CA ungranted deny; revoke reconcile deny; no sudo\n")?;
    run("systemctl", &["poweroff"])?;
    Ok(())
}
fn main() -> Result<()> {
    let result = proof();
    if let Err(error) = &result {
        // Failed QA boots must also flush their native journal before inspection.
        if fs::read_to_string("/sys/class/dmi/id/product_name")
            .is_ok_and(|v| v.trim() == "fungos-mycelium-access-validation")
        {
            eprintln!("ACCESS_PROOF_FAILED: {error}");
            let _ = private(
                "/var/lib/fungos-access-proof-error",
                error.to_string().as_bytes(),
            );
            let _ = run("systemctl", &["poweroff"]);
        }
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_wrong_guest() {
        assert!(require(false, "wrong guest").is_err());
    }
    #[test]
    fn private_path_is_not_peer_state() {
        assert_ne!(QA, "/var/lib/mycelium");
    }
}
