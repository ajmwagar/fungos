//! Retain exact-revision upstream notices omitted from published crate archives.
//! Missing evidence stays UNKNOWN; this does not approve redistribution.
use std::{fs, path::Path, process::Command};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn run(program: &str, args: &[&str]) -> Result<String> {
    let out = Command::new(program).args(args).output()?;
    if !out.status.success() {
        return Err(format!("{program}: {}", String::from_utf8_lossy(&out.stderr)).into());
    }
    Ok(String::from_utf8(out.stdout)?)
}
fn github(repository: &str) -> Option<String> {
    let path = repository
        .strip_prefix("https://github.com/")?
        .trim_end_matches('/')
        .trim_end_matches(".git");
    let parts: Vec<_> = path.split('/').collect();
    ((parts.len() == 2 || (parts.len() > 3 && parts[2] == "tree"))
        && parts[..2].iter().all(|p| {
            !p.is_empty()
                && p.bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        }))
    .then(|| parts[..2].join("/"))
}
fn notice(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
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
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 3 {
        return Err(
            "usage: upstream-notice-supplement VENDOR MISSING_PACKAGES_TSV NEW_OUTPUT".into(),
        );
    }
    let vendor = Path::new(&args[0]);
    let output = Path::new(&args[2]);
    fs::create_dir(output)?;
    fs::write(output.join("status.txt"), "incomplete\n")?;
    let mut inventory = String::from("package\trepository\trevision\tstatus\tnotice_files\n");
    for row in fs::read_to_string(&args[1])?.lines() {
        let fields: Vec<_> = row.split('\t').collect();
        if fields.first() == Some(&"package") || (fields.len() >= 5 && fields[4] != "0") {
            continue;
        }
        let package = row.split('\t').next().ok_or("package")?;
        if package.is_empty()
            || !package
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        {
            return Err("unsafe package".into());
        }
        let root = vendor.join(package);
        let repository = run(
            "sed",
            &[
                "-n",
                "s/^repository = \"\\(.*\\)\"$/\\1/p",
                root.join("Cargo.toml").to_str().ok_or("path")?,
            ],
        )?
        .lines()
        .next()
        .unwrap_or_default()
        .trim()
        .to_owned();
        let revision = run(
            "jq",
            &[
                "-r",
                ".git.sha1 // empty",
                root.join(".cargo_vcs_info.json").to_str().ok_or("path")?,
            ],
        )
        .unwrap_or_default()
        .trim()
        .to_owned();
        let mut count = 0;
        let result: Result<()> = (|| {
            let repo =
                github(&repository).ok_or("unsupported or absent exact upstream repository")?;
            if revision.len() != 40 || !revision.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err("missing exact revision".into());
            }
            let url = format!("https://api.github.com/repos/{repo}/contents?ref={revision}");
            let listing = run(
                "curl",
                &[
                    "--fail",
                    "--silent",
                    "--show-error",
                    "--location",
                    "--proto-redir",
                    "=https",
                    "--max-time",
                    "30",
                    &url,
                ],
            )?;
            let listfile = output.join(format!("{package}.json"));
            fs::write(&listfile, listing)?;
            let entries = run(
                "jq",
                &[
                    "-r",
                    ".[] | select(.type == \"file\") | [.name, .download_url] | @tsv",
                    listfile.to_str().ok_or("path")?,
                ],
            )?;
            for entry in entries.lines() {
                let (name, url) = entry.split_once('\t').ok_or("upstream file row")?;
                if !notice(name) {
                    continue;
                }
                if name.contains(['/', '\\', '\n', '\r'])
                    || !url.starts_with("https://raw.githubusercontent.com/")
                    || !url.contains(&format!("/{revision}/"))
                {
                    return Err("unbounded upstream path".into());
                }
                let text = run(
                    "curl",
                    &[
                        "--fail",
                        "--silent",
                        "--show-error",
                        "--max-time",
                        "30",
                        url,
                    ],
                )?;
                let dest = output.join(package);
                fs::create_dir_all(&dest)?;
                fs::write(dest.join(name), text)?;
                count += 1;
            }
            if count == 0 {
                return Err(
                    "no root notices at exact upstream revision; nested review required".into(),
                );
            }
            Ok(())
        })();
        let status = match result {
            Ok(()) => "retained-root-notices".to_owned(),
            Err(error) => format!("UNKNOWN: {error}").replace(['\t', '\n', '\r'], " "),
        };
        inventory.push_str(&format!(
            "{package}\t{repository}\t{revision}\t{status}\t{count}\n"
        ));
        fs::write(output.join("provenance.tsv"), &inventory)?;
    }
    fs::write(
        output.join("status.txt"),
        "collection-completed; UNKNOWN rows remain blockers\nlegal_clearance=not-asserted\n",
    )?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repository_is_bounded() {
        assert_eq!(github("https://github.com/a/b.git"), Some("a/b".into()));
        for bad in ["http://github.com/a/b", "https://github.com/a/../b"] {
            assert!(github(bad).is_none());
        }
    }
    #[test]
    fn notice_names() {
        assert!(notice("LICENSE-MIT"));
        assert!(!notice("Cargo.toml"));
    }
}
