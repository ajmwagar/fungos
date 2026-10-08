//! Linux-only qualification with disposable keys/packages and isolated APT
//! directories. Never installs anything or reads the fleet's signer/identities.
#![cfg(target_os = "linux")]
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn run(command: &mut Command) -> Output {
    let output = command.output().expect("execute qualification tool");
    assert!(
        output.status.success(),
        "tool failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove owned test scratch");
    }
}

fn apt(root: &Path, source: &Path, label: &str) -> Command {
    let work = root.join(label);
    fs::create_dir_all(work.join("state/lists/partial")).unwrap();
    fs::create_dir_all(work.join("cache/archives/partial")).unwrap();
    fs::create_dir_all(work.join("log")).unwrap();
    fs::write(work.join("status"), "").unwrap();
    let mut command = Command::new("apt-get");
    for option in [
        format!("Dir::State={}/state", work.display()),
        format!("Dir::State::status={}/status", work.display()),
        format!("Dir::Cache={}/cache", work.display()),
        format!("Dir::Log={}/log", work.display()),
        format!("Dir::Etc::sourcelist={}", source.display()),
        "Dir::Etc::sourceparts=-".into(),
        "Dir::Etc::main=-".into(),
        "Dir::Etc::parts=-".into(),
        "APT::Update::Error-Mode=any".into(),
        "Acquire::Languages=none".into(),
    ] {
        command.args(["-o", &option]);
    }
    command.arg("update");
    command
}

#[test]
fn signed_multiarch_generation_is_accepted_and_tampering_denied() {
    let scratch = run(Command::new("mktemp").arg("-d"));
    let scratch = Scratch(PathBuf::from(
        String::from_utf8(scratch.stdout).unwrap().trim(),
    ));
    let root = &scratch.0;
    let signing = root.join("signing");
    fs::create_dir(&signing).unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&signing, fs::Permissions::from_mode(0o700)).unwrap();
    run(Command::new("gpg").arg("--homedir").arg(&signing).args([
        "--batch",
        "--pinentry-mode",
        "loopback",
        "--passphrase",
        "",
        "--quick-generate-key",
        "fungOS disposable qualification (not production)",
        "ed25519",
        "sign",
        "1d",
    ]));
    let keys = run(Command::new("gpg")
        .arg("--homedir")
        .arg(&signing)
        .args(["--with-colons", "--list-keys"]));
    let keys = String::from_utf8(keys.stdout).unwrap();
    let fingerprint = keys
        .lines()
        .find(|line| line.starts_with("fpr:"))
        .unwrap()
        .split(':')
        .nth(9)
        .unwrap();
    let mut packages = Vec::new();
    for (arch, version) in [("amd64", "0.1.0"), ("amd64", "0.1.1"), ("arm64", "0.1.0")] {
        let directory = root.join(format!("fixture-{arch}-{version}"));
        fs::create_dir_all(directory.join("DEBIAN")).unwrap();
        fs::write(directory.join("DEBIAN/control"), format!("Package: fungos-test-fixture\nVersion: {version}\nArchitecture: {arch}\nMaintainer: fungOS qualification\nDescription: disposable qualification, not a fleet package\n")).unwrap();
        let package = root.join(format!("fixture-{arch}-{version}.deb"));
        run(Command::new("dpkg-deb")
            .args(["--build", "--root-owner-group"])
            .arg(directory)
            .arg(&package));
        packages.push(package);
    }
    let repository = root.join("repository");
    let mut publisher = Command::new(env!("CARGO_BIN_EXE_fungos-apt-repository"));
    publisher
        .env("GNUPGHOME", &signing)
        .arg("--write")
        .arg(&repository)
        .arg(fingerprint)
        .arg("testing")
        .args(&packages);
    run(&mut publisher);
    assert!(repository.join("READY").is_file());
    let generation = fs::read_to_string(repository.join("GENERATION")).unwrap();
    let digest = run(Command::new("sha256sum").arg(repository.join("SHA256SUMS")));
    assert_eq!(
        generation.trim(),
        String::from_utf8(digest.stdout)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap()
    );
    run(Command::new("sha256sum")
        .current_dir(&repository)
        .args(["--check", "SHA256SUMS"]));
    let release = fs::read_to_string(repository.join("dists/testing/Release")).unwrap();
    assert!(release.contains("Acquire-By-Hash: yes"));
    assert!(release.contains("Valid-Until:"));
    assert!(release.contains("NotAutomatic: yes"));
    for arch in ["amd64", "arm64"] {
        let index = repository.join(format!("dists/testing/main/binary-{arch}/Packages"));
        let digest = run(Command::new("sha256sum").arg(&index));
        let digest = String::from_utf8(digest.stdout).unwrap();
        assert_eq!(
            fs::read(&index).unwrap(),
            fs::read(
                index
                    .parent()
                    .unwrap()
                    .join("by-hash/SHA256")
                    .join(digest.split_whitespace().next().unwrap())
            )
            .unwrap()
        );
    }
    let amd64 =
        fs::read_to_string(repository.join("dists/testing/main/binary-amd64/Packages")).unwrap();
    assert!(amd64.contains("Version: 0.1.0") && amd64.contains("Version: 0.1.1"));
    // Existing output must never be overwritten.
    assert!(!publisher.output().unwrap().status.success());

    let source = root.join("source.list");
    let keyring = repository.join("fungos-archive-keyring.gpg");
    fs::write(
        &source,
        format!(
            "deb [arch=amd64,arm64 signed-by={}] file:{} testing main\n",
            keyring.display(),
            repository.display()
        ),
    )
    .unwrap();
    run(&mut apt(root, &source, "apt-good"));
    let wrong_key = root.join("wrong-key.gpg");
    fs::write(&wrong_key, "").unwrap();
    let wrong_source = root.join("wrong.list");
    fs::write(
        &wrong_source,
        format!(
            "deb [signed-by={}] file:{} testing main\n",
            wrong_key.display(),
            repository.display()
        ),
    )
    .unwrap();
    assert!(!apt(root, &wrong_source, "apt-wrong-key")
        .output()
        .unwrap()
        .status
        .success());

    // Corrupt both named and hash-addressed indexes; new APT state cannot hide
    // a bad download behind a previously cached trusted index.
    for arch in ["amd64", "arm64"] {
        let directory = repository.join(format!("dists/testing/main/binary-{arch}"));
        for entry in fs::read_dir(directory.join("by-hash/SHA256")).unwrap() {
            fs::write(entry.unwrap().path(), "corrupted index").unwrap();
        }
        fs::write(directory.join("Packages"), "corrupted index").unwrap();
        fs::write(directory.join("Packages.gz"), "corrupted index").unwrap();
    }
    assert!(!apt(root, &source, "apt-tampered")
        .output()
        .unwrap()
        .status
        .success());
    assert!(!Command::new("sha256sum")
        .current_dir(&repository)
        .args(["--check", "SHA256SUMS"])
        .output()
        .unwrap()
        .status
        .success());
}
