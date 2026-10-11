//! Prove raw boot copies against exact retained package payloads.
use std::{fs,io::Read,path::Path,process::Command};
type Result<T> = std::result::Result<T,Box<dyn std::error::Error>>;
fn digest(path:&Path)->Result<String>{let out=Command::new("sha256sum").arg(path).output()?;if !out.status.success(){return Err("hash failed".into())} Ok(String::from_utf8(out.stdout)?.split_whitespace().next().ok_or("missing digest")?.into())}
fn table_only(a:&[u8],b:&[u8])->bool{a.len()==b.len() && a.len()>=64 && a[..8]==b[..8] && a[64..]==b[64..]}
fn mbr_code(a:&[u8],b:&[u8])->bool{a.len()==432 && b.len()==432 && &a[..2]==b"ER" && a[32..]==b[32..]}
fn main()->Result<()>{
 let args:Vec<_>=std::env::args().skip(1).collect();
 if args.len()!=7{return Err("usage: media-bindings MEDIA BOOT_PACKAGE_ROOT SYSLINUX_ROOT EFI_FAT_ROOT DEBIAN_CD_SOURCE ISO NEW_BINDINGS.tsv".into())}
 let media=Path::new(&args[0]);let boot=Path::new(&args[1]);let syslinux=Path::new(&args[2]);let fat=Path::new(&args[3]);
 let pairs=[
 ("EFI/boot/bootx64.efi",boot.join("usr/lib/shim/shimx64.efi.signed"),"shim-signed=1.47 (shim=15.8-1)"),
 ("EFI/boot/grubx64.efi",boot.join("usr/lib/grub/x86_64-efi-signed/grubx64.efi.signed"),"grub-efi-amd64-signed=1+2.12+9 (grub2=2.12-9)"),
 ("boot/grub/font.pf2",boot.join("usr/share/grub/ascii.pf2"),"grub2=2.12-9"),
 ("install.amd/vmlinuz",boot.join("boot/vmlinuz-6.12.43+deb13-amd64"),"linux-signed-amd64=6.12.43+1 (linux=6.12.43-1)"),
 ("install.amd/gtk/vmlinuz",boot.join("boot/vmlinuz-6.12.43+deb13-amd64"),"linux-signed-amd64=6.12.43+1 (linux=6.12.43-1)"),
 ("install.amd/xen/vmlinuz",boot.join("boot/vmlinuz-6.12.43+deb13-amd64"),"linux-signed-amd64=6.12.43+1 (linux=6.12.43-1)"),
 ("isolinux/ldlinux.c32",syslinux.join("usr/lib/syslinux/modules/bios/ldlinux.c32"),"syslinux=3:6.04~git20190206.bf6db5b4+dfsg1-3.1"),
 ("isolinux/vesamenu.c32",syslinux.join("usr/lib/syslinux/modules/bios/vesamenu.c32"),"syslinux=3:6.04~git20190206.bf6db5b4+dfsg1-3.1"),
 ("isolinux/libcom32.c32",syslinux.join("usr/lib/syslinux/modules/bios/libcom32.c32"),"syslinux=3:6.04~git20190206.bf6db5b4+dfsg1-3.1"),
 ("isolinux/libutil.c32",syslinux.join("usr/lib/syslinux/modules/bios/libutil.c32"),"syslinux=3:6.04~git20190206.bf6db5b4+dfsg1-3.1")];
 let mut report=String::from("media_path\tsha256\tpackage_payload\texact_source_binding\tcomparison\n");
 for (name,payload,source) in pairs{
  if fs::read(media.join(name))?!=fs::read(&payload)?{return Err(format!("raw boot payload differs: {name}").into())}
  report.push_str(&format!("{name}\t{}\t{}\t{source}\tbyte-identical\n",digest(&media.join(name))?,payload.display()));
 }
 for name in ["bootx64.efi","grubx64.efi"]{
  if fs::read(fat.join(name))?!=fs::read(media.join("EFI/boot").join(name))?{return Err("EFI FAT payload differs".into())}
  report.push_str(&format!("boot/grub/efi.img:/EFI/BOOT/{name}\t{}\tEFI/boot/{name}\tverified-copy-of-row-above\tbyte-identical\n",digest(&fat.join(name))?));
 }
 let isolinux=media.join("isolinux/isolinux.bin");let original=syslinux.join("usr/lib/ISOLINUX/isolinux.bin");
 if !table_only(&fs::read(&isolinux)?,&fs::read(&original)?){return Err("isolinux differences outside audited boot information table".into())}
 report.push_str(&format!("isolinux/isolinux.bin\t{}\t{}\tsyslinux=3:6.04~git20190206.bf6db5b4+dfsg1-3.1\tidentical-except-offsets-8-through-63;core/isolinux.asm:170-185\n",digest(&isolinux)?,original.display()));
 let cd=Path::new(&args[4]);
 for entry in fs::read_dir(media.join("pics"))? {
  let entry=entry?;if !entry.file_type()?.is_file(){return Err("unexpected artwork kind".into())}
  let name=entry.file_name();let bytes=fs::read(entry.path())?;
  let candidates=[cd.join("data/pics").join(&name),cd.join("data/trixie").join(&name)];
  let matched=candidates.iter().find(|p|fs::read(p).is_ok_and(|v|v==bytes)).ok_or("artwork not byte-identical to retained Debian CD source")?;
  report.push_str(&format!("pics/{}\t{}\t{}\tdebian-cd=3.2.2\tbyte-identical\n",name.to_str().ok_or("non UTF8 artwork")?,digest(&entry.path())?,matched.display()));
 }
 let mut prefix=[0u8;432];fs::File::open(&args[5])?.read_exact(&mut prefix)?;
 let mbr=syslinux.join("usr/lib/ISOLINUX/isohdpfx.bin");
 if !mbr_code(&prefix,&fs::read(&mbr)?){return Err("isohybrid executable bytes differ outside audited 32-byte APM header".into())}
 report.push_str(&format!("ISO-system-area;digest-covers-whole-ISO\t{}\t{}\tsyslinux=3:6.04~git20190206.bf6db5b4+dfsg1-3.1\texecutable-offsets-32-through-431-identical;offsets-0-through-31-APM-header;rest-generated-partition-metadata\n",digest(Path::new(&args[5]))?,mbr.display()));
 fs::OpenOptions::new().write(true).create_new(true).open(&args[6])?;
 fs::write(&args[6],report)?;
 println!("raw boot and artwork bindings verified; config/generated ISO structures require source/build scripts");Ok(())
}
#[cfg(test)]mod tests{#[test]fn table_is_narrow(){let a=vec![0;100];let mut b=a.clone();b[8]=1;b[63]=2;assert!(super::table_only(&a,&b));b[64]=1;assert!(!super::table_only(&a,&b));assert!(!super::table_only(&[0;8],&[0;8]));}#[test]fn mbr_exceptions_do_not_cover_executable_body(){let mut a=[0;432];let b=a;a[..2].copy_from_slice(b"ER");assert!(super::mbr_code(&a,&b));a[32]=1;assert!(!super::mbr_code(&a,&b));}}
