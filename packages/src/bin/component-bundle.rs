//! Validate owner-built Debian components before an image may include them.
//! This assembles inputs; it never builds or signs somebody else's package.
use std::{collections::BTreeSet, env, fs, path::Path, process::Command};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
struct Component<'a> {
    name: &'a str,
    version: &'a str,
    arch: &'a str,
    digest: &'a str,
    file: &'a str,
    source: &'a str,
    revision: &'a str,
    license: &'a str,
}

fn hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
}

fn parse(input: &str) -> Result<Vec<Component<'_>>> {
    let mut components = Vec::new();
    let mut names = BTreeSet::new();
    let mut files = BTreeSet::new();
    for line in input
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
    {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 9 || fields.iter().any(|f| f.is_empty()) {
            return Err("component manifest requires nine nonempty tab-separated fields".into());
        }
        let c = Component {
            name: fields[0],
            version: fields[1],
            arch: fields[2],
            digest: fields[3],
            file: fields[4],
            source: fields[5],
            revision: fields[6],
            license: fields[7],
        };
        if !matches!(c.name, "mycelium" | "unibus-core") || fields[8] != "apt" {
            return Err(
                "headless-edge requires owner-built mycelium/unibus-core, exclusively APT-managed"
                    .into(),
            );
        }
        if !matches!(c.arch, "amd64" | "arm64") || !hex(c.digest, 64) || !hex(c.revision, 40) {
            return Err("invalid component architecture, SHA-256 or full source commit".into());
        }
        if !c.file.ends_with(".deb")
            || c.file.starts_with('.')
            || !c
                .file
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._+-".contains(&b))
        {
            return Err("package filename must be a plain .deb basename".into());
        }
        if !c.source.starts_with("https://")
            || c.source.contains([' ', '\r'])
            || c.license.contains('\r')
        {
            return Err(
                "source must be HTTPS and provenance must not contain control characters".into(),
            );
        }
        if !names.insert(c.name) || !files.insert(c.file) {
            return Err("duplicate component or filename".into());
        }
        components.push(c);
    }
    if names != BTreeSet::from(["mycelium", "unibus-core"]) {
        return Err("headless-edge is missing a required runtime component".into());
    }
    if components.iter().any(|c| c.arch != components[0].arch) {
        return Err("component architectures disagree".into());
    }
    Ok(components)
}

fn run(program: &str, arguments: &[&str]) -> Result<String> {
    let output = Command::new(program).args(arguments).output()?;
    if !output.status.success() {
        return Err(format!(
            "{program} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

fn verify(manifest: &str, directory: &Path) -> Result<()> {
    if !fs::symlink_metadata(directory)?.file_type().is_dir() {
        return Err("component bundle must be a real directory".into());
    }
    for c in parse(manifest)? {
        let path = directory.join(c.file);
        if !fs::symlink_metadata(&path)?.file_type().is_file() {
            return Err("package must be a regular file, not a symlink".into());
        }
        let path = path.to_str().ok_or("non-UTF8 package path")?;
        if run("sha256sum", &[path])?.split_whitespace().next() != Some(c.digest) {
            return Err(format!("{} checksum mismatch", c.name).into());
        }
        for (field, expected) in [
            ("Package", c.name),
            ("Version", c.version),
            ("Architecture", c.arch),
        ] {
            if run("dpkg-deb", &["--field", path, field])?.trim() != expected {
                return Err(format!("{} {field} mismatch", c.name).into());
            }
        }
        // Reject operational credentials and automatic maintainer-script execution.
        // Authenticated source provenance remains the release owner's responsibility.
        let control = run("dpkg-deb", &["--info", path])?;
        for script in ["preinst", "postinst", "prerm", "postrm"] {
            if control
                .lines()
                .any(|line| line.split_whitespace().last() == Some(script))
            {
                return Err(
                    "component maintainer scripts require deliberate lifecycle review".into(),
                );
            }
        }
        let listing = run("dpkg-deb", &["--contents", path])?;
        for line in listing.lines() {
            if line.contains("/.mycelium/")
                || line.contains("/ssh_host_")
                || line.contains("/first-contact.env")
                || line.contains("/.env")
                || line.contains("/credentials.env")
            {
                return Err("identity or credential state in component package".into());
            }
        }
        let required: &[&str] = match c.name {
            "mycelium" => &[
                "./usr/bin/mycelium",
                "./usr/lib/systemd/system/mycelium.service",
                "./usr/share/doc/mycelium/copyright",
            ],
            _ => &[
                "./usr/bin/unibus-router",
                "./usr/bin/unibusctl",
                "./etc/unibus/hub.json",
                "./usr/lib/systemd/system/unibus-router.service",
                "./usr/share/doc/unibus-core/copyright",
            ],
        };
        for entry in required {
            if !listing
                .lines()
                .any(|l| l.split_whitespace().last() == Some(entry))
            {
                return Err(format!("{} missing {entry}", c.name).into());
            }
        }
    }
    Ok(())
}

fn checksums(contents: &str, directory: &Path) -> Result<String> {
    let mut sums = String::new();
    for file in parse(contents)?
        .into_iter()
        .map(|c| c.file)
        .chain(std::iter::once("components.tsv"))
    {
        let path = directory.join(file);
        let digest = run("sha256sum", &[path.to_str().ok_or("non-UTF8 output path")?])?;
        sums.push_str(&format!(
            "{}  {file}\n",
            digest.split_whitespace().next().ok_or("missing SHA-256")?
        ));
    }
    Ok(sums)
}

fn main() -> Result<()> {
    let args: Vec<_> = env::args().skip(1).collect();
    match args.as_slice() {
        [mode, bundle] if mode == "verify" => {
            let directory = Path::new(bundle);
            let manifest = fs::read_to_string(directory.join("components.tsv"))?;
            verify(&manifest, directory)?;
            if fs::read_to_string(directory.join("SHA256SUMS"))? != checksums(&manifest, directory)?
            {
                return Err("component bundle checksum inventory mismatch".into());
            }
        }
        [mode, manifest, output] if mode == "prepare" => {
            let input = Path::new(manifest);
            let mut contents = fs::read_to_string(input)?;
            if !contents.ends_with('\n') {
                contents.push('\n');
            }
            let source = input.parent().ok_or("manifest has no parent")?;
            verify(&contents, source)?;
            let destination = Path::new(output);
            fs::create_dir(destination)?; // Never overwrite an existing generation.
            for component in parse(&contents)? {
                fs::copy(
                    source.join(component.file),
                    destination.join(component.file),
                )?;
            }
            fs::write(destination.join("components.tsv"), &contents)?;
            verify(&contents, destination)?;
            fs::write(
                destination.join("SHA256SUMS"),
                checksums(&contents, destination)?,
            )?;
        }
        _ => {
            return Err(
                "usage: component-bundle prepare INPUT.tsv NEW_DIRECTORY | verify BUNDLE_DIRECTORY"
                    .into(),
            )
        }
    }
    println!("verified owner-built headless-edge components (APT ownership)");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn manifest() -> String {
        ["mycelium", "unibus-core"].map(|name| format!("{name}\t0.1.0\tamd64\t{}\t{name}.deb\thttps://github.com/owner/{name}\t{}\tMIT OR Apache-2.0\tapt", "a".repeat(64), "b".repeat(40))).join("\n")
    }
    #[test]
    fn accepts_required_inputs() {
        let input = manifest();
        let parsed = parse(&input).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].license, "MIT OR Apache-2.0");
        assert!(parsed[0].source.starts_with("https://"));
    }
    #[test]
    fn rejects_missing_component() {
        assert!(parse(manifest().lines().next().unwrap()).is_err());
    }
    #[test]
    fn rejects_dual_ownership() {
        assert!(parse(&manifest().replace("\tapt", "\tnative")).is_err());
    }
    #[test]
    fn rejects_path_escape() {
        assert!(parse(&manifest().replace("mycelium.deb", "../mycelium.deb")).is_err());
    }
    #[test]
    fn rejects_cross_architecture() {
        assert!(parse(&manifest().replacen("\tamd64", "\tarm64", 1)).is_err());
    }
    #[test]
    fn rejects_unpinned_source() {
        assert!(parse(&manifest().replace(&"b".repeat(40), "master")).is_err());
    }
    #[test]
    fn rejects_invalid_digest() {
        assert!(parse(&manifest().replace(&"a".repeat(64), &"a".repeat(63))).is_err());
        assert!(parse(&manifest().replace(&"a".repeat(64), &"A".repeat(64))).is_err());
    }
    #[test]
    fn rejects_missing_provenance() {
        assert!(parse(&manifest().replace("MIT OR Apache-2.0", "")).is_err());
        assert!(parse(&manifest().replace("https://", "file://")).is_err());
    }
    #[test]
    fn rejects_duplicate_component() {
        let input = manifest();
        assert!(parse(&format!("{input}\n{}", input.lines().next().unwrap())).is_err());
    }
}
