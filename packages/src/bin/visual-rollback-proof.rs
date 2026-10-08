//! Offline, explicitly marked disposable visual QEMU guest only.
use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    thread,
    time::Duration,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const CLI: &str = "/var/lib/mycelium/bin/mycelium";
const STATE: &str = "/var/lib/fungos-visual-proof";
fn disposable(root: &str, product: &str) -> bool {
    root.trim() == "0" && product.trim() == "fungos-visual-rollback-only"
}
fn run(program: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(program).args(args).output()?;
    if !output.status.success() {
        return Err(format!(
            "{program}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?)
}
fn ready() -> Result<()> {
    println!("FUNGOS_VISUAL_WAITING_FOR_OWNER_READINESS");
    for _ in 0..90 {
        let current =
            run(CLI, &["software", "reconcile", "--dry-run", "--json"]).is_ok_and(|report| {
                report.contains("\"state\": \"current\"")
                    && !report.contains("\"state\": \"drifted\"")
            });
        if current
            && run(
                "systemctl",
                &[
                    "is-active",
                    "mycelium",
                    "canvas-compositor",
                    "canvas",
                    "canvas-edge",
                    "unibus-router",
                ],
            )
            .is_ok()
            && owner("inspect\n").is_ok()
        {
            return Ok(());
        }
        thread::sleep(Duration::from_secs(2));
    }
    Err("edge services did not become ready".into())
}
fn owner(input: &str) -> Result<String> {
    let mut child = Command::new("/usr/local/bin/canvasctl")
        .env("XDG_RUNTIME_DIR", "/run/fungos-canvas")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("no owner stdin")?
        .write_all(input.as_bytes())?;
    let output = child.wait_with_output()?;
    let text = String::from_utf8(output.stdout)?;
    if !output.status.success() || text.contains("ERR ") || !text.contains("OK ") {
        return Err(format!("Canvas owner failed: {text}").into());
    }
    Ok(text)
}
fn retained() -> Result<String> {
    run(
        "sha256sum",
        &[
            "/etc/machine-id",
            "/var/lib/mycelium/pki/node.pem",
            "/var/lib/fungos-edge/workspaces.json",
        ],
    )
}
fn rollback(package: &str, unit: &str) -> Result<()> {
    println!("FUNGOS_VISUAL_INJECTING_FAILED_RELEASE {package}");
    let current = format!("/var/lib/mycelium/software/{package}/current");
    let before = fs::read_link(&current)?;
    let before_digest = run("sha256sum", &[&format!("{current}/{package}")])?;
    run(
        CLI,
        &[
            "packages",
            "publish",
            "--name",
            package,
            "--binary",
            "/opt/fungos-proof/broken",
            "--signing-key",
            "/opt/fungos-proof/signer",
            "--version",
            "0.9.99",
            "--channel",
            "visual-rollback-only",
            "--write",
            "--json",
        ],
    )?;
    match run(
        CLI,
        &[
            "software",
            "activate",
            package,
            "--channel",
            "visual-rollback-only",
            "--write",
            "--json",
        ],
    ) {
        Err(error) if error.to_string().contains("previous version restored") => (),
        result => return Err(format!("expected verified rollback, got {result:?}").into()),
    }
    if fs::read_link(&current)? != before
        || run("sha256sum", &[&format!("{current}/{package}")])? != before_digest
    {
        return Err("rollback did not restore exact prior bytes and link".into());
    }
    run("systemctl", &["is-active", unit])?;
    if Path::new(&format!(
        "/var/lib/mycelium/software/{package}/activation-pending.json"
    ))
    .exists()
    {
        return Err("rollback left recovery pending".into());
    }
    println!("FUNGOS_VISUAL_ROLLBACK_VERIFIED {package}");
    Ok(())
}
fn main() -> Result<()> {
    if !disposable(
        &run("id", &["-u"])?,
        &fs::read_to_string("/sys/class/dmi/id/product_name")?,
    ) {
        return Err("refusing test outside explicitly marked disposable guest".into());
    }
    std::env::set_var("MYCELIUM_HOME", "/var/lib/mycelium");
    std::env::set_var("MYCELIUM_NO_AUTOSTART", "1");
    ready()?;
    if Path::new(STATE).exists() {
        if retained()? != fs::read_to_string(STATE)? {
            return Err("identity/workspace changed across reboot".into());
        }
        println!("FUNGOS_VISUAL_REBOOT_RETAINED");
        println!("{}", owner("inspect\n")?);
        // Leave a bounded interval for read-only host framebuffer capture.
        thread::sleep(Duration::from_secs(60));
        run("systemctl", &["--no-block", "poweroff"])?;
        return Ok(());
    }
    owner("bg-image /opt/fungos-proof/undergrowth.png\nws-save dock\n")?;
    let baseline = retained()?;
    rollback("canvas", "canvas.service")?;
    rollback("unibus-router", "unibus-router.service")?;
    ready()?;
    if retained()? != baseline {
        return Err("identity/workspace changed during rollback".into());
    }
    println!("{}", owner("inspect\n")?);
    fs::write(STATE, baseline)?;
    println!("FUNGOS_VISUAL_ROLLBACK_STATE_RETAINED");
    run("systemctl", &["--no-block", "reboot"])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::disposable;
    #[test]
    fn requires_root_and_explicit_disposable_marker() {
        assert!(disposable("0\n", "fungos-visual-rollback-only\n"));
        assert!(!disposable("1000", "fungos-visual-rollback-only"));
        assert!(!disposable("0", "fungos-edge-rebuilt-01"));
        assert!(!disposable("0", "fungos-visual-rollback-only-other"));
    }
}
