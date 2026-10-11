//! Hash retained Debian source closures; this inventories evidence, not legal approval.
use std::{collections::{BTreeMap,BTreeSet},fs,path::{Path,PathBuf},process::Command};
type Result<T>=std::result::Result<T,Box<dyn std::error::Error>>;
fn sha(path:&Path)->Result<String>{let out=Command::new("sha256sum").arg(path).output()?;if !out.status.success(){return Err("hash failed".into())}Ok(String::from_utf8(out.stdout)?.split_whitespace().next().ok_or("missing digest")?.into())}
fn dscs(root:&Path,out:&mut Vec<PathBuf>)->Result<()>{for entry in fs::read_dir(root)?{let entry=entry?;let kind=entry.file_type()?;if kind.is_dir(){dscs(&entry.path(),out)?}else if kind.is_file() && entry.path().extension().is_some_and(|v|v=="dsc"){out.push(entry.path())}}Ok(())}
fn sums(data:&str)->Result<Vec<(String,u64,String)>>{
 let mut active=false;let mut out=Vec::new();
 for line in data.lines(){if line=="Checksums-Sha256:"{active=true;continue}if active && !line.starts_with(' '){break}if active{let values:Vec<_>=line.split_whitespace().collect();if values.len()!=3 || values[0].len()!=64 || !values[0].bytes().all(|b|b.is_ascii_hexdigit()) || values[2].contains('/') || values[2].contains('\\') || values[2]==".."{return Err("unsafe DSC checksum row".into())}out.push((values[0].into(),values[1].parse()?,values[2].into()));}}
 if out.is_empty(){return Err("missing DSC sha256 files".into())}Ok(out)
}
fn main()->Result<()>{
 let args:Vec<_>=std::env::args().skip(1).collect();if args.len()<3{return Err("usage: source-manifest REQUIREMENTS.tsv NEW_MANIFEST.tsv AUTHENTICATED_SOURCE_ROOT...".into())}
 if Path::new(&args[1]).exists(){return Err("refusing existing manifest".into())}
 let mut required=BTreeSet::new();for line in fs::read_to_string(&args[0])?.lines(){let fields:Vec<_>=line.split('\t').collect();if fields.len()!=3{return Err("invalid requirements".into())}required.insert((fields[0].to_owned(),fields[1].to_owned()));}
 let mut found=BTreeMap::new();let mut report=String::from("source\tversion\tarchive\tsha256\tbytes\tretained_path\n");
 for directory in &args[2..]{let mut paths=Vec::new();dscs(Path::new(directory),&mut paths)?;paths.sort();for path in paths{
  let data=fs::read_to_string(&path)?;let field=|key:&str|data.lines().find_map(|l|l.strip_prefix(key));
  let name=field("Source: ").ok_or("DSC missing source")?.to_owned();let version=field("Version: ").ok_or("DSC missing version")?.to_owned();
  let key=(name.clone(),version.clone());let digest=sha(&path)?;
  if let Some(old)=found.get(&key){if old!=&digest{return Err("conflicting exact source DSCs".into())}continue}
  let parent=path.parent().ok_or("no parent")?;let files=sums(&data)?;
  for (expected,size,filename) in &files{let file=parent.join(filename);let stat=fs::symlink_metadata(&file)?;if !stat.is_file() || stat.len()!=*size || sha(&file)?!=*expected{return Err(format!("source archive mismatch: {}",file.display()).into())}}
  report.push_str(&format!("{name}\t{version}\t{}\t{digest}\t{}\t{}\n",path.file_name().ok_or("no filename")?.to_str().ok_or("nonUTF8")?,fs::metadata(&path)?.len(),path.display()));
  for (digest,size,filename) in files{report.push_str(&format!("{name}\t{version}\t{filename}\t{digest}\t{size}\t{}\n",parent.join(&filename).display()));}
  found.insert(key,digest);
 }}
 let missing:Vec<_>=required.iter().filter(|key|!found.contains_key(*key)).collect();if !missing.is_empty(){for (name,version) in &missing{eprintln!("MISSING {name} {version}")}return Err(format!("{} exact source closures absent; no manifest emitted",missing.len()).into())}
 fs::OpenOptions::new().write(true).create_new(true).open(&args[1])?;fs::write(&args[1],report)?;
 println!("verified {} required exact sources; retained {} exact source DSC closures; does not establish full image/native license coverage",required.len(),found.len());Ok(())
}
#[cfg(test)]mod tests{#[test]fn checksums_reject_path_escape(){let valid=format!("Checksums-Sha256:\n {} 1 foo.tar.xz\nFiles:\n", "a".repeat(64));assert_eq!(super::sums(&valid).unwrap().len(),1);assert!(super::sums(&valid.replace("foo.tar.xz","../foo")).is_err());assert!(super::sums("Checksums-Sha256:\n bad 1 foo").is_err());}}
