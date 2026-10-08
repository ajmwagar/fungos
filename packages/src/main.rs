//! Publish immutable signed APT repository generations using Debian's tools.
//! Package construction stays with the component's owning project.
use std::{env, fs, path::Path, process::Command};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn run(root: &Path, program: &str, args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "{program} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(output.stdout)
}

fn fingerprint(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn sha256(root: &Path, path: &str) -> Result<String> {
    let output = String::from_utf8(run(root, "sha256sum", &["--", path])?)?;
    let digest = output.split_whitespace().next().ok_or("missing SHA256")?;
    if digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid SHA256 output".into());
    }
    Ok(digest.to_ascii_lowercase())
}

fn inventory(root: &Path, directory: &Path, paths: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_dir() {
            inventory(root, &entry.path(), paths)?;
        } else if kind.is_file() {
            paths.push(
                entry
                    .path()
                    .strip_prefix(root)?
                    .to_str()
                    .ok_or("non-UTF8 path")?
                    .to_owned(),
            );
        } else {
            return Err("generation must contain only regular files and directories".into());
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 6 || args[1] != "--write" {
        return Err("usage: fungos-apt-repository --write NEW_OUTPUT SIGNER_FINGERPRINT testing|stable PACKAGE.deb... (GNUPGHOME selects signer)".into());
    }
    let root = Path::new(&args[2]);
    if !fingerprint(&args[3]) || !matches!(args[4].as_str(), "testing" | "stable") {
        return Err("expected full signing fingerprint and testing|stable suite".into());
    }
    // Refuse existing generations. Errors leave an unpublished directory for inspection.
    fs::create_dir(root)?;
    let root = root.canonicalize()?;
    fs::create_dir(root.join("pool"))?;
    let mut architectures = std::collections::BTreeSet::new();
    for argument in &args[5..] {
        let package = Path::new(argument).canonicalize()?;
        if !fs::symlink_metadata(argument)?.is_file()
            || package.extension().and_then(|s| s.to_str()) != Some("deb")
        {
            return Err("packages must be regular non-symlink .deb files".into());
        }
        let arch = String::from_utf8(run(
            &root,
            "dpkg-deb",
            &[
                "-f",
                package.to_str().ok_or("non-UTF8 path")?,
                "Architecture",
            ],
        )?)?;
        let arch = arch.trim();
        if !matches!(arch, "amd64" | "arm64") {
            return Err(format!("unsupported architecture {arch}").into());
        }
        architectures.insert(arch.to_owned());
        let name = String::from_utf8(run(
            &root,
            "dpkg-deb",
            &["-f", package.to_str().ok_or("non-UTF8 path")?, "Package"],
        )?)?;
        let version = String::from_utf8(run(
            &root,
            "dpkg-deb",
            &["-f", package.to_str().ok_or("non-UTF8 path")?, "Version"],
        )?)?;
        let name = name.trim();
        let version = version.trim();
        if ![name, version].iter().all(|value| {
            !value.is_empty()
                && value
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b".+-:~".contains(&b))
        }) {
            return Err("unsafe package name/version".into());
        }
        let destination = root
            .join("pool")
            .join(format!("{name}_{version}_{arch}.deb"));
        let mut output = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(destination)?;
        std::io::copy(&mut fs::File::open(package)?, &mut output)?;
        output.sync_all()?;
    }
    for arch in &architectures {
        let directory = root.join(format!("dists/{}/main/binary-{arch}", args[4]));
        fs::create_dir_all(&directory)?;
        let packages = run(
            &root,
            "dpkg-scanpackages",
            &["--multiversion", "--arch", arch, "pool"],
        )?;
        if !packages.windows(9).any(|window| window == b"Package: ") {
            return Err(format!("empty package index for {arch}").into());
        }
        fs::write(directory.join("Packages"), packages)?;
        let plain = format!("dists/{}/main/binary-{arch}/Packages", args[4]);
        fs::write(
            directory.join("Packages.gz"),
            run(&root, "gzip", &["-n", "-c", "--", &plain])?,
        )?;
        let hash_directory = directory.join("by-hash/SHA256");
        fs::create_dir_all(&hash_directory)?;
        for index in ["Packages", "Packages.gz"] {
            let relative = format!("dists/{}/main/binary-{arch}/{index}", args[4]);
            fs::copy(
                root.join(&relative),
                hash_directory.join(sha256(&root, &relative)?),
            )?;
        }
    }
    let distribution = format!("dists/{}", args[4]);
    let suite = format!("APT::FTPArchive::Release::Suite={}", args[4]);
    let codename = format!("APT::FTPArchive::Release::Codename={}", args[4]);
    let arches = format!(
        "APT::FTPArchive::Release::Architectures={}",
        architectures.into_iter().collect::<Vec<_>>().join(" ")
    );
    let expiration = String::from_utf8(run(&root, "date", &["-u", "-R", "-d", "+7 days"])?)?;
    // Some apt-ftparchive versions ignore the Valid-Until config option.
    // Emit this required signed field explicitly rather than silently omitting it.
    let expiration = format!("Valid-Until: {}\n", expiration.trim());
    let release = run(
        &root,
        "apt-ftparchive",
        &[
            "-o",
            "APT::FTPArchive::Release::Origin=fungOS",
            "-o",
            "APT::FTPArchive::Release::Components=main",
            "-o",
            "APT::FTPArchive::Release::Acquire-By-Hash=yes",
            "-o",
            "APT::FTPArchive::Release::NotAutomatic=yes",
            "-o",
            "APT::FTPArchive::Release::ButAutomaticUpgrades=yes",
            "-o",
            &suite,
            "-o",
            &codename,
            "-o",
            &arches,
            "release",
            &distribution,
        ],
    )?;
    let mut release_with_expiry = expiration.into_bytes();
    release_with_expiry.extend(release);
    fs::write(
        root.join(&distribution).join("Release"),
        release_with_expiry,
    )?;
    let release = format!("{distribution}/Release");
    let signed = format!("{distribution}/InRelease");
    run(
        &root,
        "gpg",
        &[
            "--batch",
            "--local-user",
            &args[3],
            "--digest-algo",
            "SHA256",
            "--output",
            &signed,
            "--clearsign",
            &release,
        ],
    )?;
    let public_key = run(&root, "gpg", &["--batch", "--export", &args[3]])?;
    if public_key.is_empty() {
        return Err("signer exported no public key".into());
    }
    let keyring = root.join("fungos-archive-keyring.gpg");
    fs::write(&keyring, public_key)?;
    run(
        &root,
        "gpgv",
        &[
            "--keyring",
            keyring.to_str().ok_or("non-UTF8 keyring")?,
            &signed,
        ],
    )?;
    let mut paths = Vec::new();
    inventory(&root, &root, &mut paths)?;
    paths.sort();
    let mut manifest = String::new();
    for path in paths {
        manifest.push_str(&format!("{}  {path}\n", sha256(&root, &path)?));
    }
    fs::write(root.join("SHA256SUMS"), manifest)?;
    run(&root, "sha256sum", &["--check", "SHA256SUMS"])?;
    let generation_id = sha256(&root, "SHA256SUMS")?;
    fs::write(root.join("GENERATION"), format!("{generation_id}\n"))?;
    fs::write(
        root.join("READY"),
        format!("generation={generation_id}\nSigned generation verified; publish this whole directory atomically.\n"),
    )?;
    println!("{}", root.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_fingerprint_only() {
        assert!(fingerprint(&"a".repeat(40)));
        assert!(fingerprint(&"F".repeat(64)));
        assert!(!fingerprint("12345678"));
        assert!(!fingerprint(&"z".repeat(40)));
        assert!(!fingerprint("--default-key"));
    }
    #[test]
    fn failed_command_is_not_success() {
        assert!(run(Path::new("/"), "false", &[]).is_err());
    }
}
