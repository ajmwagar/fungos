//! Retain package-owned notices from exact media archives without installing them.
use std::{fs, path::{Path, PathBuf}, process::{Command, Stdio}};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn output(program: &str, args: &[&str]) -> Result<String> {
    let out = Command::new(program).args(args).env("LC_ALL", "C").output()?;
    if !out.status.success() { return Err(format!("{program}: {}", String::from_utf8_lossy(&out.stderr)).into()); }
    Ok(String::from_utf8(out.stdout)?)
}
fn text(path: &Path) -> Result<&str> { path.to_str().ok_or_else(|| "non UTF8 path".into()) }
fn archives(root: &Path, items: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() { archives(&entry.path(), items)?; }
        else if entry.file_type()?.is_file() && matches!(entry.path().extension().and_then(|x|x.to_str()), Some("deb"|"udeb")) { items.push(entry.path()); }
    }
    Ok(())
}
fn notice(path: &str) -> bool {
    let path = path.strip_prefix("./").unwrap_or(path);
    !path.split('/').any(|p| p == ".." || p.is_empty()) &&
        (path.starts_with("usr/share/common-licenses/") ||
         (path.starts_with("usr/share/doc/") && path.ends_with("/copyright")))
}
fn tar(archive: &Path, args: &[&str]) -> Result<String> {
    let mut package = Command::new("dpkg-deb").args(["--fsys-tarfile", text(archive)?]).stdout(Stdio::piped()).stderr(Stdio::inherit()).spawn()?;
    let result = Command::new("tar").args(args).stdin(package.stdout.take().ok_or("missing pipe")?).output()?;
    let package_status = package.wait()?;
    if !result.status.success() || !package_status.success() { return Err(format!("notice extraction failed: {}", String::from_utf8_lossy(&result.stderr)).into()); }
    Ok(String::from_utf8(result.stdout)?)
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len()!=2 { return Err("usage: media-notices EXTRACTED_ISO NEW_NOTICE_DIR".into()); }
    let media = Path::new(&args[0]); let target=Path::new(&args[1]);
    fs::create_dir(target)?;
    let mut packages=Vec::new(); archives(media,&mut packages)?; packages.sort();
    let mut manifest=String::from("archive\tarchive_sha256\tpackage\tversion\tsource\tnotices\n");
    for (index, archive) in packages.iter().enumerate() {
        let relative=text(archive.strip_prefix(media)?)?;
        let digest=output("sha256sum", &[text(archive)?])?.split_whitespace().next().ok_or("missing digest")?.to_owned();
        let package=output("dpkg-deb", &["-f",text(archive)?,"Package"])?.trim().to_owned();
        let version=output("dpkg-deb", &["-f",text(archive)?,"Version"])?.trim().to_owned();
        let source=output("dpkg-deb", &["-f",text(archive)?,"Source"])?.trim().to_owned();
        let names:Vec<_>=tar(archive,&["-tf","-"])?.lines().filter(|x|notice(x)).map(str::to_owned).collect();
        let directory=target.join(format!("{index:04}-{package}")); fs::create_dir(&directory)?;
        if !names.is_empty() {
            let mut extract=vec!["-xf","-","--no-same-owner","--no-same-permissions","-C",text(&directory)?];
            extract.extend(names.iter().map(String::as_str)); tar(archive,&extract)?;
        }
        manifest.push_str(&format!("{relative}\t{digest}\t{package}\t{version}\t{source}\t{}\n",names.len()));
        fs::write(directory.join("archive-origin.tsv"),format!("{relative}\t{digest}\t{package}\t{version}\t{source}\n"))?;
    }
    fs::write(target.join("packages.tsv"),manifest)?;
    println!("retained notices from {} archives; zero-notice packages require source-owned notices",packages.len());
    Ok(())
}
#[cfg(test)] mod tests {
    #[test] fn only_bounded_package_notice_paths() {
        assert!(super::notice("./usr/share/doc/firmware-iwlwifi/copyright"));
        assert!(super::notice("./usr/share/common-licenses/GPL-2"));
        for value in ["../../copyright","/usr/share/doc/a/copyright","./usr/share/doc/../copyright","./etc/passwd","./usr/share/doc/a/"] { assert!(!super::notice(value)); }
    }
}
