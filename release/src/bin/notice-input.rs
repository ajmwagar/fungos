//! Stage raw notice inputs with explicit artifact binding; never claim legal clearance.
//! This deliberately emits incomplete coverage until unresolved notices are reviewed.
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn raw_notice(path: &Path) -> bool {
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
    .any(|part| name.contains(part))
}
fn hash(path: &Path) -> Result<String> {
    let out = Command::new("sha256sum").arg(path).output()?;
    if !out.status.success() {
        return Err("sha256sum failed".into());
    }
    Ok(String::from_utf8(out.stdout)?
        .split_whitespace()
        .next()
        .ok_or("digest")?
        .into())
}
// Bound argv size and avoid starting one process per small notice file.
fn hashes(paths: &[PathBuf]) -> Result<Vec<String>> {
    let mut result = Vec::new();
    for batch in paths.chunks(64) {
        let out = Command::new("sha256sum").arg("--").args(batch).output()?;
        if !out.status.success() {
            return Err("notice batch hashing failed".into());
        }
        let text = String::from_utf8(out.stdout)?;
        let rows: Vec<_> = text.lines().collect();
        if rows.len() != batch.len() {
            return Err("notice batch hash count mismatch".into());
        }
        for (row, path) in rows.into_iter().zip(batch) {
            let (digest, name) = row.split_once("  ").ok_or("hash row")?;
            if digest.len() != 64
                || !digest.bytes().all(|b| b.is_ascii_hexdigit())
                || name != path.to_str().ok_or("non-UTF8 notice path")?
            {
                return Err("notice batch hash identity mismatch".into());
            }
            result.push(digest.to_owned());
        }
    }
    Ok(result)
}
fn visit(root: &Path, here: &Path, files: &mut BTreeMap<PathBuf, PathBuf>) -> Result<()> {
    for entry in fs::read_dir(here)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            return Err("notice symlinks forbidden".into());
        }
        if kind.is_dir() {
            visit(root, &entry.path(), files)?;
        } else if kind.is_file() {
            if !raw_notice(&entry.path()) {
                continue;
            }
            let relative = entry.path().strip_prefix(root)?.to_owned();
            if relative
                .to_string_lossy()
                .contains(['\t', '\n', '\r', '\\'])
            {
                return Err("unsafe path".into());
            }
            files.insert(relative, entry.path());
        } else {
            return Err("notice nonregular input".into());
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_raw_notice_names() {
        assert!(raw_notice(Path::new("LICENSE.txt")));
        assert!(!raw_notice(Path::new("provenance.tsv")));
        assert!(!raw_notice(Path::new("crate.json")));
    }
    #[test]
    fn empty_batch_has_no_hashes() {
        assert!(hashes(&[]).unwrap().is_empty());
    }
    #[test]
    fn batch_matches_individual_hashes_across_boundary() {
        let root = std::env::temp_dir().join(format!(
            "fungos-notice-batch-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        let path = root.join("notice with spaces.txt");
        fs::write(&path, b"original notice bytes").unwrap();
        let expected = hash(&path).unwrap();
        let paths = vec![path.clone(); 65];
        assert_eq!(hashes(&paths).unwrap(), vec![expected.clone(); 65]);
        fs::write(&path, b"changed notice bytes").unwrap();
        assert_ne!(hashes(&[path]).unwrap(), vec![expected]);
        assert!(hashes(&[root.join("missing")]).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() < 5 {
        return Err(
            "usage: notice-input NEW_OUTPUT SOURCE_COMMIT BINARY_SHA LOCK_SHA NOTICE_DIRECTORY..."
                .into(),
        );
    }
    for (value, length) in [(&args[1], 40), (&args[2], 64), (&args[3], 64)] {
        if value.len() != length || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("invalid provenance digest/revision".into());
        }
    }
    let output = Path::new(&args[0]);
    let mut planned = BTreeMap::new();
    for (i, input) in args[4..].iter().enumerate() {
        let mut files = BTreeMap::new();
        visit(Path::new(input), Path::new(input), &mut files)?;
        for (relative, source) in files {
            let destination = PathBuf::from(format!("notices/input-{i}")).join(relative);
            planned.insert(destination, source);
        }
    }
    // Collection errors cannot create a plausible output. Later I/O failure may
    // leave evidence files, but metadata is written last and always incomplete.
    let originals: Vec<_> = planned.values().cloned().collect();
    let expected_hashes = hashes(&originals)?;
    fs::create_dir(output)?;
    let mut inventory = String::from("sha256\tpath\n");
    let mut copied = Vec::new();
    for (destination, source) in &planned {
        let path = output.join(&destination);
        fs::create_dir_all(path.parent().ok_or("parent")?)?;
        fs::copy(&source, &path)?;
        copied.push(path);
    }
    for (((destination, _), expected), actual) in planned
        .iter()
        .zip(expected_hashes)
        .zip(hashes(&copied)?)
    {
        if actual != expected {
            return Err("notice changed during collection".into());
        }
        inventory.push_str(&format!("{actual}\t{}\n", destination.display()));
    }
    fs::write(output.join("inventory.tsv"), inventory)?;
    fs::write(output.join("metadata.txt"),format!("schema_version=1\nsource_commit={}\nexecutable_sha256={}\ncargo_lock_sha256={}\ninventory_sha256={}\ncoverage=incomplete\nlegal_clearance=not-asserted\n",args[1],args[2],args[3],hash(&output.join("inventory.tsv"))?))?;
    println!("notice input staged; incomplete coverage must reject packaging");
    Ok(())
}
