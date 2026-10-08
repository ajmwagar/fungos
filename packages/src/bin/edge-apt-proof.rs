//! Never run on a fleet node: disposable QEMU edge-package qualification only.
use std::{fs, path::Path, process::Command};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
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
fn update(package: &str, version: &str, executable: &str, should_pass: bool) -> Result<()> {
    let output = command(
        "/opt/fungos-apt/mycelium",
        &["software", "apt", "plan", package, version, executable],
    )?;
    let digest = output
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("\"digest\": \"")
                .and_then(|s| s.strip_suffix("\","))
        })
        .ok_or("missing plan digest")?;
    let plan = format!("/opt/fungos-apt/{package}-{version}.plan.json");
    fs::write(&plan, &output)?;
    // Explicit digest rejects substitution without making a package change.
    if command(
        "/opt/fungos-apt/mycelium",
        &[
            "software",
            "apt",
            "apply",
            &plan,
            &"0".repeat(64),
            "--write",
        ],
    )
    .is_ok()
    {
        return Err("wrong digest was accepted".into());
    }
    let applied = command(
        "/opt/fungos-apt/mycelium",
        &["software", "apt", "apply", &plan, digest, "--write"],
    );
    match (should_pass, applied) {
        (true, Ok(_)) => println!("FUNGOS_EDGE_APT_UPGRADE_VERIFIED {package} {version}"),
        (false, Err(error))
            if error
                .to_string()
                .contains("previous package restored and health verified") =>
        {
            println!("FUNGOS_EDGE_APT_FAILED_UPDATE_ROLLBACK_VERIFIED {package}")
        }
        (_, result) => return Err(format!("unexpected update outcome: {result:?}").into()),
    }
    if Path::new("/var/lib/mycelium/apt/activation-pending.json").exists() {
        return Err("verified result left recovery pending".into());
    }
    Ok(())
}
fn main() -> Result<()> {
    if command("id", &["-u"])?.trim() != "0"
        || fs::read_to_string("/sys/class/dmi/id/product_name")?.trim()
            != "fungos-edge-apt-validation"
    {
        return Err("refusing operation outside explicitly marked disposable QEMU guest".into());
    }
    std::env::set_var("MYCELIUM_HOME", "/var/lib/mycelium");
    if Path::new("/var/lib/fungos-edge-apt-proof").exists() {
        for (package, unit) in [
            ("unibus-core", "unibus-router"),
            ("canvas-linux", "canvas-compositor"),
        ] {
            if command("dpkg-query", &["-W", "-f=${Version}", package])? != "0.1.0+aptqa2" {
                return Err("package version did not persist across reboot".into());
            }
            command("systemctl", &["is-active", unit])?;
            let executable = if package == "unibus-core" {
                "/usr/bin/unibus-router"
            } else {
                "/usr/bin/canvas-linux"
            };
            command(
                "/opt/fungos-apt/mycelium",
                &["software", "apt", "health", package, executable],
            )?;
        }
        println!("FUNGOS_EDGE_APT_REBOOT_VERIFIED");
        command("systemctl", &["--no-block", "poweroff"])?;
        return Ok(());
    }
    command("apt-get", &["update", "--error-on=any"])?;
    let config = fs::read("/etc/unibus/hub.json")?;
    update(
        "unibus-core",
        "0.1.0+aptqa2",
        "/usr/bin/unibus-router",
        true,
    )?;
    update(
        "unibus-core",
        "0.1.0+aptqa3",
        "/usr/bin/unibus-router",
        false,
    )?;
    update(
        "canvas-linux",
        "0.1.0+aptqa2",
        "/usr/bin/canvas-linux",
        true,
    )?;
    update(
        "canvas-linux",
        "0.1.0+aptqa3",
        "/usr/bin/canvas-linux",
        false,
    )?;
    if fs::read("/etc/unibus/hub.json")? != config {
        return Err("operator conffile changed".into());
    }
    fs::write("/var/lib/fungos-edge-apt-proof", "verified\n")?;
    println!("FUNGOS_EDGE_APT_CONFFILE_VERIFIED");
    command("sync", &[])?;
    command("systemctl", &["--no-block", "reboot"])?;
    Ok(())
}
