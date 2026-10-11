//! Materialize only verified source-manifest rows, without duplicating cache bytes.
use std::{collections::BTreeSet, fs, path::Path, process::Command};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn component(s: &str) -> bool {
    !s.is_empty()
        && s.len() < 250
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b".+:~_-".contains(&b))
        && s != "."
        && s != ".."
}
fn hash(path: &Path) -> Result<String> {
    let out = Command::new("sha256sum").arg(path).output()?;
    if !out.status.success() {
        return Err("hash failed".into());
    }
    Ok(String::from_utf8(out.stdout)?
        .split_whitespace()
        .next()
        .ok_or("missing digest")?
        .into())
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: source-stage VERIFIED_SOURCE_MANIFEST.tsv NEW_SOURCE_DIR".into());
    }
    let root = Path::new(&args[1]);
    fs::create_dir(root)?;
    let mut manifest = String::from("source\tversion\tarchive\tsha256\tbytes\tbundle_path\n");
    let mut seen = BTreeSet::new();
    let mut count = 0;
    for (index, line) in fs::read_to_string(&args[0])?.lines().enumerate() {
        if index == 0 {
            if line != "source\tversion\tarchive\tsha256\tbytes\tretained_path" {
                return Err("wrong source manifest header".into());
            }
            continue;
        }
        let row: Vec<_> = line.split('\t').collect();
        if row.len() != 6 || !row[..3].iter().all(|v| component(v)) || row[3].len() != 64 {
            return Err("unsafe source row".into());
        }
        let source = Path::new(row[5]);
        let stat = fs::symlink_metadata(source)?;
        if !stat.is_file() || stat.len() != row[4].parse::<u64>()? || hash(source)? != row[3] {
            return Err("source staging bytes mismatch".into());
        }
        let relative = Path::new(row[0]).join(row[1]).join(row[2]);
        if !seen.insert(relative.clone()) {
            return Err("duplicate bundle path".into());
        }
        let destination = root.join(&relative);
        fs::create_dir_all(destination.parent().ok_or("no parent")?)?;
        fs::hard_link(source, &destination)?;
        manifest.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\n",
            row[0],
            row[1],
            row[2],
            row[3],
            row[4],
            relative.display()
        ));
        count += 1;
    }
    if count == 0 {
        return Err("empty source staging".into());
    }
    fs::write(root.join("SOURCE-MANIFEST.tsv"), manifest)?;
    println!("staged {count} verified source files; owner source/notices and release signing remain separate");
    Ok(())
}
#[cfg(test)]
mod tests {
    #[test]
    fn path_components_are_bounded() {
        for v in ["linux", "1:15.1.01-1", "foo.orig.tar.xz"] {
            assert!(super::component(v))
        }
        for v in ["", ".", "..", "../foo", "a/b", "/foo", "a\tb"] {
            assert!(!super::component(v))
        }
    }
}
