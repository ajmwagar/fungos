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
fn built_using(data: &str) -> Result<BTreeSet<(String, String)>> {
    let mut required = BTreeSet::new();
    for stanza in data.split("\n\n") {
        if stanza
            .lines()
            .any(|line| line.starts_with("Status: ") && line != "Status: install ok installed")
        {
            continue;
        }
        let mut field = String::new();
        let mut active = false;
        for line in stanza.lines().chain(std::iter::once("")) {
            if active && line.starts_with([' ', '\t']) {
                field.push(' ');
                field.push_str(line.trim());
                continue;
            }
            if active {
                for dependency in field.split(',').map(str::trim) {
                    let (name, revision) = dependency
                        .split_once(" (= ")
                        .ok_or("nonexact Built-Using source")?;
                    let revision = revision
                        .strip_suffix(')')
                        .ok_or("malformed Built-Using source")?;
                    if !identity(name) || !identity(revision) {
                        return Err("unsafe Built-Using source".into());
                    }
                    required.insert((name.to_owned(), revision.to_owned()));
                }
                field.clear();
            }
            let value = line
                .strip_prefix("Built-Using: ")
                .or_else(|| line.strip_prefix("Static-Built-Using: "));
            active = value.is_some_and(|v| !v.is_empty());
            if let Some(value) = value {
                field.push_str(value);
            }
        }
    }
    Ok(required)
}
fn status_origin(status: Option<&str>) -> Option<&'static str> {
    match status {
        Some("install ok installed") => Some("installed-status"),
        Some(value) if value.ends_with(" ok config-files") => {
            Some("retained-config-source-conservative")
        }
        _ => None,
    }
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
                    "Built-Using".into(),
                    "Static-Built-Using".into(),
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
            for (name, revision) in built_using(&data)? {
                output.insert((name, revision, "iso-built-using".into()));
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
    for (name, revision) in built_using(&fs::read_to_string(initrd_status)?)? {
        required.insert((name, revision, "installer-built-using".into()));
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
    unstable: bool,
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
    for snapshot in BTreeSet::from([snapshot, installer_snapshot]) {
        let archives: &[(&str, &str)] = if unstable {
            &[("debian", "sid")]
        } else {
            &[
                ("debian", "trixie"),
                ("debian", "trixie-updates"),
                ("debian-security", "trixie-security"),
            ]
        };
        for (archive, suite) in archives {
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
        Some("built-using") if args.len() == 3 => {
            if Path::new(&args[2]).exists(){return Err("refusing existing source requirements".into())}
            let mut result=String::new();
            for (name,revision) in built_using(&fs::read_to_string(&args[1])?)? {result.push_str(&format!("{name}\t{revision}\tinstalled-built-using\n"));}
            fs::write(&args[2],result)?;Ok(())
        },
        Some("status-sources") if args.len() == 3 => {
            if Path::new(&args[2]).exists(){return Err("refusing existing source requirements".into())}
            let data=fs::read_to_string(&args[1])?;let mut required=BTreeSet::new();
            for stanza in data.split("\n\n") {
                let status=stanza.lines().find_map(|line|line.strip_prefix("Status: "));
                let Some(origin)=status_origin(status) else {continue};
                for (name,revision) in control(stanza)? {if name!="mycelium"{required.insert((name,revision,origin));}}
            }
            let mut result=String::new();for (name,revision,origin) in required{result.push_str(&format!("{name}\t{revision}\t{origin}\n"));}
            fs::write(&args[2],result)?;Ok(())
        },
        Some("fetch") if args.len() == 6 => fetch(Path::new(&args[1]), Path::new(&args[2]), Path::new(&args[3]), &args[4], &args[5], false),
        Some("fetch-sid") if args.len() == 5 => fetch(Path::new(&args[1]), Path::new(&args[2]), Path::new(&args[3]), &args[4], &args[4], true),
        Some("resolve-initrd") if args.len() == 4 => resolve_initrd(Path::new(&args[1]), Path::new(&args[2]), Path::new(&args[3])),
        _ => Err("usage: source-closure inventory INSTALLED.tsv EXTRACTED_ISO INITRD_DPKG_STATUS NEW_REQUIREMENTS.tsv | fetch REQUIREMENTS.tsv NEW_PRIVATE_CACHE DEBIAN_ARCHIVE_KEYRING INSTALLED_SNAPSHOT INSTALLER_SNAPSHOT".into())
    }
}
// The installer's compact status often omits Source. Resolve those binaries
// against the authenticated source indexes retained by our isolated APT fetch.
fn resolve_initrd(requirements: &Path, lists: &Path, output: &Path) -> Result<()> {
    if output.exists() {
        return Err("refusing existing resolved requirements".into());
    }
    let mut mappings = BTreeSet::new();
    for entry in fs::read_dir(lists)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() || !entry.file_name().to_string_lossy().contains("Sources")
        {
            continue;
        }
        let data = run(
            "/usr/lib/apt/apt-helper",
            &[
                "cat-file".into(),
                entry.path().to_str().ok_or("non UTF8 index")?.into(),
            ],
            None,
        )?;
        for stanza in data.split("\n\n") {
            let field = |name: &str| folded_field(stanza, name);
            if let (Some(name), Some(version), Some(binaries)) =
                (field("Package"), field("Version"), field("Binary"))
            {
                for binary in binaries.split(',').map(str::trim) {
                    mappings.insert((binary.to_owned(), version.to_owned(), name.to_owned()));
                }
            }
        }
    }
    let mut resolved = BTreeSet::new();
    for line in fs::read_to_string(requirements)?.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 3 {
            return Err("requirements must have three fields".into());
        }
        if fields[0] == "mycelium" {
            continue;
        } // Separate owner source closure.
        if fields[2] != "installer-initrd" {
            resolved.insert(line.to_owned());
            continue;
        }
        // Audited Debian d-i20250803+deb13u1 build/Makefile609-611 emits
        // targetstring-BUILD_DATE; debian/rules derives DATE from changelog
        // VERSION and passes it as BUILD_DATE. This one observed ISO target
        // is admitted ONLY if that exact suffix also exists in signed Sources.
        let version = if fields[0] == "debian-installer" {
            audited_installer_version(fields[1])
                .ok_or("unrecognized/unaudited installer build marker")?
        } else {
            bin_nmu_source_version(fields[1])
        };
        let names: BTreeSet<_> = mappings
            .iter()
            .filter(|(binary, revision, _)| binary == fields[0] && revision == version)
            .map(|(_, _, name)| name)
            .collect();
        if names.len() != 1 {
            return Err(format!(
                "unresolved/ambiguous initrd source {} {}",
                fields[0], fields[1]
            )
            .into());
        }
        resolved.insert(format!(
            "{}\t{version}\tinstaller-initrd",
            names.first().unwrap()
        ));
    }
    fs::write(
        output,
        resolved.into_iter().collect::<Vec<_>>().join("\n") + "\n",
    )?;
    Ok(())
}
fn bin_nmu_source_version(version: &str) -> &str {
    match version.rsplit_once("+b") {
        Some((source, suffix))
            if !suffix.is_empty() && suffix.bytes().all(|b| b.is_ascii_digit()) =>
        {
            source
        }
        _ => version,
    }
}
fn folded_field(stanza: &str, name: &str) -> Option<String> {
    let prefix = format!("{name}: ");
    let mut lines = stanza.lines();
    let mut value = lines
        .find_map(|line| line.strip_prefix(&prefix))
        .map(str::to_owned)?;
    for line in lines {
        if !line.starts_with([' ', '\t']) {
            break;
        }
        value.push(' ');
        value.push_str(line.trim());
    }
    Some(value)
}
fn audited_installer_version(marker: &str) -> Option<&str> {
    // Deliberately not a heuristic for future daily/custom installer builds.
    matches!(
        marker,
        "cdrom-isolinux-20250803+deb13u1" | "cdrom-gtk-20250803+deb13u1"
    )
    .then_some("20250803+deb13u1")
}
#[cfg(test)]
mod tests {
    #[test]
    fn status_classification_does_not_call_residual_configs_installed() {
        assert_eq!(
            super::status_origin(Some("install ok installed")),
            Some("installed-status")
        );
        assert_eq!(
            super::status_origin(Some("deinstall ok config-files")),
            Some("retained-config-source-conservative")
        );
        assert_eq!(super::status_origin(Some("purge ok not-installed")), None);
    }
    #[test]
    fn includes_exact_folded_built_using() {
        assert_eq!(super::built_using("Built-Using: linux (= 6.12.38-1),\n font (= 2:1)\nStatic-Built-Using: libc (= 3)\n").unwrap(), std::collections::BTreeSet::from([("linux".into(),"6.12.38-1".into()),("font".into(),"2:1".into()),("libc".into(),"3".into())]));
        assert!(super::built_using("Built-Using: foo (>= 1)\n").is_err());
        assert!(super::built_using(
            "Status: deinstall ok config-files\nBuilt-Using: linux (= 1)\n"
        )
        .unwrap()
        .is_empty());
    }
    #[test]
    fn admits_only_audited_installer_marker() {
        assert_eq!(
            super::audited_installer_version("cdrom-isolinux-20250803+deb13u1"),
            Some("20250803+deb13u1")
        );
        assert_eq!(
            super::audited_installer_version("cdrom-gtk-20250803+deb13u1"),
            Some("20250803+deb13u1")
        );
        assert_eq!(
            super::audited_installer_version("netboot-20250803+deb13u1"),
            None
        );
        assert_eq!(
            super::audited_installer_version("cdrom-isolinux-20261010"),
            None
        );
    }
    #[test]
    fn retains_folded_binary_names_in_signed_source_indexes() {
        assert_eq!(
            super::folded_field(
                "Package: linux\nBinary: headers,\n ata-modules, kernel\nVersion: 1\n",
                "Binary"
            ),
            Some("headers, ata-modules, kernel".into())
        );
    }
    #[test]
    fn strips_only_final_numeric_bin_nmu() {
        assert_eq!(super::bin_nmu_source_version("1:1.2+b3"), "1:1.2");
        assert_eq!(super::bin_nmu_source_version("1+bfix"), "1+bfix");
    }
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
