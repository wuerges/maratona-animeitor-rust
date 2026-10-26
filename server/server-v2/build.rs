use std::{env, path::PathBuf, process::Command};

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=ANIMEITOR_GIT_COMMIT");
    // Track both detached HEADs and branch updates, including packed refs.
    for name in [
        Some("HEAD".to_owned()),
        Some("packed-refs".to_owned()),
        git(&["symbolic-ref", "-q", "HEAD"]),
    ]
    .into_iter()
    .flatten()
    {
        if let Some(path) = git(&["rev-parse", "--git-path", &name]) {
            println!("cargo:rerun-if-changed={}", PathBuf::from(path).display());
        }
    }
    let revision = env::var("ANIMEITOR_GIT_COMMIT")
        .ok()
        .filter(|s| !s.is_empty());
    let revision = revision.or_else(|| git(&["rev-parse", "HEAD"]));
    if let Some(revision) = revision {
        assert!(
            matches!(revision.len(), 40 | 64) && revision.bytes().all(|b| b.is_ascii_hexdigit()),
            "ANIMEITOR_GIT_COMMIT must be a full hexadecimal Git commit hash"
        );
        println!(
            "cargo:rustc-env=ANIMEITOR_BUILD_COMMIT={}",
            revision[..12].to_ascii_lowercase()
        );
    }
}
