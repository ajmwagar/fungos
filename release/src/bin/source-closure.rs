//! Inventory exact Debian source requirements; fetch through isolated signed APT.
//! This is not a declaration that arbitrary media have complete source coverage.
use std::{collections::BTreeSet, fs, path::Path, process::Command};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn run(program: &str, args: &[String], cwd: Option<&Path>) -> Result<String> {
    let mut command = Command::new(program);
    command
        .args(args)
        .env("LC_ALL", "C")
        .env_remove("APT_CONFIG");
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let out = command.output()?;
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
fn identity(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 200
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".+:~_-".contains(&b))
        && value.as_bytes()[0].is_ascii_alphanumeric()
}
fn source(package: &str, version: &str, field: Option<&str>) -> Result<(String, String)> {
    let (name, revision) = if let Some(field) = field.filter(|s| !s.is_empty()) {
        if let Some((name, rest)) = field.split_once(" (") {
            (
                name,
                rest.strip_suffix(')')
                    .ok_or("malformed Debian source version")?,
            )
        } else {
            (field, version)
        }
    } else {
        (package, version)
    };
    if !identity(name) || !identity(revision) {
        return Err("unsafe Debian source identity".into());
    }
    Ok((name.into(), revision.into()))
}
fn control(data: &str) -> Result<Vec<(String, String)>> {
    let mut result = Vec::new();
    for stanza in data.split("\n\n") {
        let value = |key: &str| {
            stanza
                .lines()
                .find_map(|l| l.strip_prefix(&format!("{key}: ")))
        };
        if let (Some(package), Some(version)) = (value("Package"), value("Version")) {
            result.push(source(package, version, value("Source"))?);
        }
    }
    Ok(result)
}
fn media(directory: &Path, output: &mut BTreeSet<(String, String, String)>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            media(&entry.path(), output)?;
        } else if kind.is_file()
            && matches!(
                entry.path().extension().and_then(|v| v.to_str()),
                Some("deb" | "udeb")
            )
        {
            let data = run(
                "dpkg-deb",
                &[
                    "-f".into(),
                    entry.path().to_str().ok_or("non UTF8 archive")?.into(),
                    "Package".into(),
                    "Version".into(),
                    "Source".into(),
                ],
                None,
            )?;
            let records = control(&data)?;
            if records.len() != 1 {
                return Err("Debian archive missing exact source identity".into());
            }
            for (name, revision) in records {
                output.insert((name, revision, "iso-package".into()));
            }
        }
    }
    Ok(())
}
fn inventory(
    installed: &Path,
    extracted: &Path,
    initrd_status: &Path,
    output: &Path,
) -> Result<()> {
    if output.exists() {
        return Err("refusing existing source-requirements file".into());
    }
    let mut required = BTreeSet::new();
    for line in fs::read_to_string(installed)?.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 5 {
            return Err("installed inventory must have five exact Debian fields".into());
        }
        let name = if fields[2].is_empty() {
            fields[0]
        } else {
            fields[2]
        };
        let revision = if fields[3].is_empty() {
            fields[1]
        } else {
            fields[3]
        };
        if name == "mycelium" {
            continue;
        } // Owner archive/vendor closure is separate.
        if !identity(name) || !identity(revision) {
            return Err("invalid installed source identity".into());
        }
        required.insert((name.to_owned(), revision.to_owned(), "installed".into()));
    }
    media(extracted, &mut required)?;
    for (name, revision) in control(&fs::read_to_string(initrd_status)?)? {
        required.insert((name, revision, "installer-initrd".into()));
    }
    if required.is_empty() {
        return Err("empty Debian source requirements".into());
    }
    let mut text = String::new();
    for (name, revision, origin) in required {
        text.push_str(&format!("{name}\t{revision}\t{origin}\n"));
    }
    fs::write(output, text)?;
    println!("inventoried package+initrd source requirements; bootloader/firmware and owner closure still require explicit coverage");
    Ok(())
}
fn fetch(
    requirements: &Path,
    root: &Path,
    keyring: &Path,
    snapshot: &str,
    installer_snapshot: &str,
) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if !fs::symlink_metadata(keyring)?.is_file() {
        return Err("Debian archive keyring must be a regular authenticated input".into());
    }
    for value in [snapshot, installer_snapshot] {
        if value.len() != 16
            || !value
                .bytes()
                .all(|b| b.is_ascii_digit() || b == b'T' || b == b'Z')
            || &value[8..9] != "T"
            || &value[15..] != "Z"
        {
            return Err("snapshot must use YYYYMMDDTHHMMSSZ".into());
        }
    }
    fs::create_dir(root)?;
    fs::set_permissions(root, fs::Permissions::from_mode(0o700))?;
    for dir in ["lists/partial", "cache/archives/partial", "sources"] {
        fs::create_dir_all(root.join(dir))?;
    }
    let key = root.join("debian-archive-keyring.gpg");
    fs::copy(keyring, &key)?;
    let config = root.join("sources.list");
    let mut sources = String::new();
    // Current installed snapshot plus the upstream13.1 netinst release window.
    // Exact source versions are selected from authenticated indexes, never latest.
    for snapshot in [snapshot, installer_snapshot] {
        for (archive, suite) in [
            ("debian", "trixie"),
            ("debian", "trixie-updates"),
            ("debian-security", "trixie-security"),
        ] {
            sources.push_str(&format!("deb-src [signed-by={}] https://snapshot.debian.org/archive/{archive}/{snapshot}/ {suite} main contrib non-free-firmware\n", key.display()));
        }
    }
    fs::write(&config, sources)?;
    let options = vec![
        "-o".into(),
        "Dir::Etc::main=-".into(),
        "-o".into(),
        "Dir::Etc::parts=-".into(),
        "-o".into(),
        format!("Dir::Etc::sourcelist={}", config.display()),
        "-o".into(),
        "Dir::Etc::sourceparts=-".into(),
        "-o".into(),
        format!("Dir::State::Lists={}", root.join("lists").display()),
        "-o".into(),
        format!("Dir::Cache={}", root.join("cache").display()),
        "-o".into(),
        "APT::Get::List-Cleanup=0".into(),
        "-o".into(),
        "Acquire::Check-Valid-Until=false".into(),
        "-o".into(),
        "Acquire::Languages=none".into(),
        "-o".into(),
        "Acquire::Retries=2".into(),
    ];
    // Expiry is explicitly disabled only for pinned historical source archives.
    // Signature/hash authentication remains mandatory. Host APT is not changed.
    let mut update = options.clone();
    update.extend(["update".into(), "--error-on=any".into()]);
    run("apt-get", &update, Some(root))?;
    let mut pairs = BTreeSet::new();
    for line in fs::read_to_string(requirements)?.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 3 || !identity(fields[0]) || !identity(fields[1]) {
            return Err("invalid source requirements".into());
        }
        pairs.insert((fields[0].to_owned(), fields[1].to_owned()));
    }
    fs::copy(requirements, root.join("requirements.tsv"))?;
    let mut coverage = String::new();
    let mut missing = String::new();
    for (name, revision) in pairs {
        let directory = root.join("sources").join(format!("{name}-{revision}"));
        fs::create_dir(&directory)?;
        let mut args = options.clone();
        args.extend([
            "source".into(),
            "--download-only".into(),
            "--only-source".into(),
            format!("{name}={revision}"),
        ]);
        match run("apt-get", &args, Some(&directory)) {
            Ok(log) => {
                fs::write(directory.join("APT-ADMISSION.txt"), log)?;
                coverage.push_str(&format!("{name}\t{revision}\tdownloaded-via-signed-apt\n"));
            }
            Err(error) => {
                fs::write(directory.join("MISSING.txt"), error.to_string())?;
                missing.push_str(&format!(
                    "{name}\t{revision}\tunavailable-or-fetch-failed\n"
                ));
            }
        }
        fs::write(root.join("coverage.tsv"), &coverage)?;
        fs::write(root.join("missing.tsv"), &missing)?;
        println!(
            "source {name} {revision}: {}",
            if directory.join("MISSING.txt").exists() {
                "MISSING"
            } else {
                "downloaded"
            }
        );
    }
    if !missing.is_empty() {
        return Err(
            "source closure incomplete; inspect pinned missing.tsv (no complete marker written)"
                .into(),
        );
    }
    println!("requested Debian package sources fetched; bootloader/firmware/license/owner source closure is still separate");
    Ok(())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("inventory") if args.len() == 5 => inventory(Path::new(&args[1]), Path::new(&args[2]), Path::new(&args[3]), Path::new(&args[4])),
        Some("fetch") if args.len() == 6 => fetch(Path::new(&args[1]), Path::new(&args[2]), Path::new(&args[3]), &args[4], &args[5]),
        _ => Err("usage: source-closure inventory INSTALLED.tsv EXTRACTED_ISO INITRD_DPKG_STATUS NEW_REQUIREMENTS.tsv | fetch REQUIREMENTS.tsv NEW_PRIVATE_CACHE DEBIAN_ARCHIVE_KEYRING INSTALLED_SNAPSHOT INSTALLER_SNAPSHOT".into())
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn source_bin_nmu_version_is_exact() {
        assert_eq!(
            super::source("foo", "1.2+b3", Some("foo (1.2)")).unwrap(),
            ("foo".into(), "1.2".into())
        );
        assert_eq!(
            super::source("foo-bin", "1.2", Some("foo")).unwrap(),
            ("foo".into(), "1.2".into())
        );
        assert!(super::source("foo", "1", Some("--bad")).is_err());
    }
    #[test]
    fn parses_control_sources_not_binary_names() {
        assert_eq!(
            super::control(
                "Package: libfoo\nVersion: 1.2+b3\nSource: foo (1.2)\n\nPackage: bar\nVersion: 2\n"
            )
            .unwrap(),
            vec![("foo".into(), "1.2".into()), ("bar".into(), "2".into())]
        );
    }
}
