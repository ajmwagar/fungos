//! Exact-release source/checksum and notice inventory, not legal clearance.
//! Uses Cargo's metadata and registry checksum files; retains full sources for
//! bundled/file-local exceptions rather than inventing a flattened SPDX claim.
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn command(
    program: &str,
    args: &[&str],
    input: Option<&str>,
    cwd: Option<&Path>,
) -> Result<String> {
    let mut c = Command::new(program);
    c.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(cwd) = cwd {
        c.current_dir(cwd);
    }
    if input.is_some() {
        c.stdin(Stdio::piped());
    }
    let mut child = c.spawn()?;
    if let Some(input) = input {
        child
            .stdin
            .take()
            .ok_or("stdin")?
            .write_all(input.as_bytes())?;
    }
    let out = child.wait_with_output()?;
    if !out.status.success() {
        return Err(format!("{program}: {}", String::from_utf8_lossy(&out.stderr)).into());
    }
    Ok(String::from_utf8(out.stdout)?)
}
fn digest(path: &Path) -> Result<String> {
    Ok(command(
        "sha256sum",
        &[path.to_str().ok_or("UTF8 path")?],
        None,
        None,
    )?
    .split_whitespace()
    .next()
    .ok_or("missing digest")?
    .into())
}
fn quoted(line: &str, key: &str) -> Option<String> {
    line.strip_prefix(&format!("{key} = \""))?
        .strip_suffix('"')
        .map(str::to_owned)
}
fn lock(data: &str) -> Result<BTreeMap<String, String>> {
    let mut result = BTreeMap::new();
    for package in data.split("[[package]]").skip(1) {
        let get = |key| package.lines().find_map(|line| quoted(line, key));
        if get("source").is_some_and(|v| v.starts_with("registry+")) {
            let name = get("name").ok_or("name")?;
            let version = get("version").ok_or("version")?;
            let checksum = get("checksum").ok_or("checksum")?;
            if !checksum.bytes().all(|b| b.is_ascii_hexdigit()) || checksum.len() != 64 {
                return Err("invalid package checksum".into());
            }
            if result
                .insert(format!("{name}-{version}"), checksum)
                .is_some()
            {
                return Err("duplicate registry package".into());
            }
        }
    }
    if result.is_empty() {
        return Err("no registry sources".into());
    }
    Ok(result)
}
fn safe_relative(path: &str) -> bool {
    !path.is_empty()
        && !path.contains(['\n', '\r', '\t', '\\'])
        && !Path::new(path).is_absolute()
        && Path::new(path)
            .components()
            .all(|p| matches!(p, std::path::Component::Normal(_)))
}
fn notice(path: &Path) -> bool {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_ascii_lowercase();
    [
        "license",
        "licence",
        "copying",
        "copyright",
        "notice",
        "authors",
    ]
    .iter()
    .any(|v| name.contains(v))
}
fn attribution_header(path: &Path) -> Result<bool> {
    let mut prefix = Vec::new();
    fs::File::open(path)?.take(8192).read_to_end(&mut prefix)?;
    let Ok(text) = std::str::from_utf8(&prefix) else {
        return Ok(false);
    };
    let text = text.to_ascii_lowercase();
    Ok([
        "copyright",
        "spdx-license-identifier",
        "permission is hereby granted",
        "redistribution and use",
    ]
    .iter()
    .any(|marker| text.contains(marker)))
}
fn retained_attribution(bytes: &[u8]) -> Vec<u8> {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return bytes.to_vec();
    };
    let lower = text.to_ascii_lowercase();
    let later_marker = |end: usize| {
        [
            "copyright",
            "spdx-license-identifier",
            "permission is hereby granted",
            "redistribution and use",
        ]
        .iter()
        .any(|marker| lower[end..].contains(marker))
    };
    let marker = [
        "copyright",
        "spdx-license-identifier",
        "permission is hereby granted",
        "redistribution and use",
    ]
    .iter()
    .filter_map(|m| lower.find(m))
    .min();
    let Some(marker) = marker else {
        return bytes.to_vec();
    };
    // Only extract a complete leading C/Rust block. Ambiguous file-local or
    // later statements retain the original file instead of guessing boundaries.
    let trimmed = text.trim_start();
    let start = text.len() - trimmed.len();
    if trimmed.starts_with("/*") {
        if let Some(end) = text[start + 2..].find("*/").map(|end| start + 2 + end + 2) {
            if marker < end && !later_marker(end) {
                return bytes[start..end].to_vec();
            }
        }
    }
    if let Some(prefix) = ["//", "#", ";", "--"]
        .iter()
        .find(|prefix| trimmed.starts_with(**prefix))
    {
        let mut end = start;
        for line in text[start..].split_inclusive('\n') {
            if !line.trim().is_empty() && !line.trim_start().starts_with(prefix) {
                break;
            }
            end += line.len();
        }
        if marker < end && !later_marker(end) {
            return bytes[start..end].to_vec();
        }
    }
    bytes.to_vec()
}
fn files(root: &Path, here: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(here)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            return Err(format!("symlink source {}", entry.path().display()).into());
        }
        if kind.is_dir() {
            files(root, &entry.path(), output)?;
        } else if kind.is_file() {
            output.push(entry.path().strip_prefix(root)?.to_owned());
        } else {
            return Err("nonregular source input".into());
        }
    }
    output.sort();
    Ok(())
}
fn collect(
    label: &str,
    root: &Path,
    output: &Path,
    inventory: &mut String,
    all_hashes: bool,
) -> Result<usize> {
    let mut listed = Vec::new();
    files(root, root, &mut listed)?;
    let mut count = 0;
    for relative in listed {
        if notice(&relative) {
            let destination = output.join("notices").join(label).join(&relative);
            fs::create_dir_all(destination.parent().ok_or("parent")?)?;
            fs::copy(root.join(&relative), destination)?;
            count += 1;
        } else if attribution_header(&root.join(&relative))? {
            // Retain the entire original file rather than truncating an ambiguous
            // header or inventing its boundary. This is conservative evidence,
            // not a statement that every detected string is a legal obligation.
            let retained = PathBuf::from("source-attributions")
                .join(&relative)
                .with_extension(format!(
                    "{}.NOTICE",
                    relative.extension().unwrap_or_default().to_string_lossy()
                ));
            let destination = output.join("notices").join(label).join(&retained);
            fs::create_dir_all(destination.parent().ok_or("parent")?)?;
            fs::write(
                &destination,
                retained_attribution(&fs::read(root.join(&relative))?),
            )?;
            inventory.push_str(&format!(
                "{label}\t{}\t{}\n",
                retained.display(),
                digest(&destination)?
            ));
            count += 1;
        }
        if all_hashes || notice(&relative) {
            inventory.push_str(&format!(
                "{}\t{}\t{}\n",
                label,
                relative.display(),
                digest(&root.join(&relative))?
            ));
        }
    }
    Ok(count)
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() == 3 && args[0] == "compact" {
        let root = Path::new(&args[1]);
        let output = Path::new(&args[2]);
        let mut listed = Vec::new();
        files(root, root, &mut listed)?;
        fs::create_dir(output)?;
        let mut inventory = String::from("sha256\tpath\n");
        for relative in listed {
            let source = root.join(&relative);
            let destination = output.join(&relative);
            fs::create_dir_all(destination.parent().ok_or("parent")?)?;
            if relative
                .components()
                .any(|p| p.as_os_str() == "source-attributions")
            {
                fs::write(&destination, retained_attribution(&fs::read(&source)?))?;
            } else {
                fs::copy(&source, &destination)?;
            }
            inventory.push_str(&format!(
                "{}\t{}\n",
                digest(&destination)?,
                relative.display()
            ));
        }
        fs::write(output.join("compact-inventory.tsv"), inventory)?;
        println!("compact attribution evidence retained; not legal clearance");
        return Ok(());
    }
    if !(6..=8).contains(&args.len()) {
        return Err(
            "usage: owner-license-audit OWNER_SOURCE VENDOR RUST_SRC RUST_DOC ZIG_LIB NEW_OUTPUT [RUST_LLVM_LIBUNWIND [SUPPLEMENTAL_NOTICES]]"
                .into(),
        );
    }
    let source = Path::new(&args[0]);
    let vendor = Path::new(&args[1]);
    let output = Path::new(&args[5]);
    fs::create_dir(output)?;
    fs::write(
        output.join("audit-status.txt"),
        "incomplete\nlegal_clearance=not-asserted\n",
    )?;
    let registry = lock(&fs::read_to_string(source.join("Cargo.lock"))?)?;
    let mut crates = String::from(
        "package\tlock_archive_sha256\tdeclared_license\tlicense_file\tnotice_files\n",
    );
    let mut notices = String::from("component\tfile\tsha256\n");
    let metadata = command(
        "cargo",
        &[
            "metadata",
            "--offline",
            "--locked",
            "--all-features",
            "--format-version",
            "1",
            "--manifest-path",
            source.join("Cargo.toml").to_str().ok_or("manifest")?,
        ],
        None,
        None,
    )?;
    let rows = command("jq", &["-r", ".packages[] | [.name, .version, (.license // \"UNKNOWN\"), (.license_file // \"\")] | @tsv"], Some(&metadata), None)?;
    let declared = rows
        .lines()
        .map(|l| {
            let fields: Vec<_> = l.split('\t').collect();
            (
                format!("{}-{}", fields[0], fields[1]),
                fields[2..].join("\t"),
            )
        })
        .collect::<BTreeMap<_, _>>();
    for (package, archive_sha) in &registry {
        let root = vendor.join(package);
        if !root.is_dir() {
            return Err(format!("missing exact vendor package {package}").into());
        }
        let check = root.join(".cargo-checksum.json");
        let package_sha = command(
            "jq",
            &["-r", ".package", check.to_str().ok_or("checksum")?],
            None,
            None,
        )?;
        if package_sha.trim() != archive_sha {
            return Err(format!("package checksum mismatch {package}").into());
        }
        let file_checks = command(
            "jq",
            &[
                "-r",
                ".files | to_entries[] | [.value, .key] | @tsv",
                check.to_str().ok_or("checksum")?,
            ],
            None,
            None,
        )?;
        let mut check_input = String::new();
        for row in file_checks.lines() {
            let (sha, path) = row.split_once('\t').ok_or("file checksum row")?;
            if !safe_relative(path)
                || sha.len() != 64
                || !sha.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err("unsafe checksum path/hash".into());
            }
            if !fs::symlink_metadata(root.join(path))?.is_file() {
                return Err("checksum target not a regular file".into());
            }
            check_input.push_str(&format!("{sha}  {path}\n"));
        }
        command(
            "sha256sum",
            &["--strict", "--check", "--status"],
            Some(&check_input),
            Some(&root),
        )?;
        let count = collect(package, &root, output, &mut notices, false)?;
        let license = declared
            .get(package)
            .ok_or_else(|| format!("registry source absent from locked metadata: {package}"))?;
        crates.push_str(&format!("{package}\t{archive_sha}\t{license}\t{count}\n"));
    }
    // Retain completed registry evidence even if a later runtime input fails.
    fs::write(output.join("registry.tsv"), &crates)?;
    // Full runtime sources are retained at these input paths. Inventory every
    // file, preserving notices separately; don't infer all files share one license.
    for (label, path) in [
        ("rust-1.98.1-library", &args[2]),
        ("rust-1.98.1-doc", &args[3]),
        ("zig-0.14.1-lib", &args[4]),
    ] {
        collect(label, Path::new(path), output, &mut notices, true)?;
    }
    if let Some(path) = args.get(6) {
        collect(
            "rust-llvm-libunwind",
            Path::new(path),
            output,
            &mut notices,
            true,
        )?;
    }
    if let Some(path) = args.get(7) {
        collect(
            "supplemental-upstream",
            Path::new(path),
            output,
            &mut notices,
            true,
        )?;
    }
    collect("mycelium-owner", source, output, &mut notices, false)?;
    fs::write(output.join("registry.tsv"), crates)?;
    fs::write(output.join("source-files.tsv"), notices)?;
    fs::write(output.join("audit-receipt.txt"), format!("registry_packages_verified={}\nlock_sha256={}\nstatus=source-integrity-and-notice-inventory-only\nlegal_clearance=not-asserted\n", registry.len(), digest(&source.join("Cargo.lock"))?))?;
    fs::write(
        output.join("audit-status.txt"),
        "inventory-completed\nlegal_clearance=not-asserted\n",
    )?;
    println!(
        "verified {} locked registry sources; source/notice inventory {}",
        registry.len(),
        output.display()
    );
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paths_are_bounded() {
        for bad in ["../x", "/x", "x/../y", "x\ny", "x\\y"] {
            assert!(!safe_relative(bad));
        }
        assert!(safe_relative("src/file.rs"));
    }
    #[test]
    fn notice_selection_preserves_filename_notices() {
        assert!(notice(Path::new("LICENSE-MIT")));
        assert!(!notice(Path::new("source.rs")));
    }
    #[test]
    fn attribution_retains_complete_block_or_full_ambiguous_source() {
        let source = b" /* Copyright A\n permission is hereby granted\n */\nint x;";
        assert_eq!(
            retained_attribution(source),
            b"/* Copyright A\n permission is hereby granted\n */"
        );
        let later = b"int x; /* Copyright A */";
        assert_eq!(retained_attribution(later), later);
        let multiple = b"/* Copyright A */\nint x; /* Copyright B */";
        assert_eq!(retained_attribution(multiple), multiple);
        let unfinished = b"/* Copyright A";
        assert_eq!(retained_attribution(unfinished), unfinished);
        assert_eq!(
            retained_attribution(b"// Copyright A\n// MIT terms\n\nfn main() {}"),
            b"// Copyright A\n// MIT terms\n\n"
        );
    }
    #[test]
    fn lock_ignores_workspace_and_rejects_bad_checksum() {
        assert!(lock("[[package]]\nname = \"local\"\nversion = \"1\"\n").is_err());
        let data = format!("[[package]]\nname = \"a\"\nversion = \"1\"\nsource = \"registry+x\"\nchecksum = \"{}\"\n", "a".repeat(64));
        assert_eq!(lock(&data).unwrap().len(), 1);
    }
}
