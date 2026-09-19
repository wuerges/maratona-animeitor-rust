//! Exercise actual stdin, diagnostics, and process exit codes without a deployment.
use std::{
    io::Write,
    process::{Command, Stdio},
};
fn run(args: &[&str], input: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_animeitor-admin"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
#[test]
fn help_and_usage_do_not_require_configuration() {
    let help = run(&["--help"], "");
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    assert!(text.contains("revelation-urls"));
    assert!(text.contains("teams"));
    let bad = run(&["timer", "set", "e", "--seconds", "wrong"], "");
    assert_eq!(bad.status.code(), Some(2));
}
#[test]
fn stdin_is_read_and_errors_stay_on_stderr() {
    let nonexistent = std::env::temp_dir().join(format!(
        "animeitor-admin-no-config-{}.toml",
        std::process::id()
    ));
    assert!(!nonexistent.exists());
    let args = [
        "--json",
        "--server-config",
        nonexistent.to_str().unwrap(),
        "events",
        "update",
        "e",
        "--file",
        "-",
    ];
    let valid = run(&args, r#"{"time_seconds":-60}"#);
    assert_eq!(valid.status.code(), Some(1));
    assert!(valid.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&valid.stderr).unwrap();
    assert!(
        error["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("reading")
    ); // Input accepted; fails only at config load.
    let invalid = run(&args, "not JSON");
    assert_eq!(invalid.status.code(), Some(1));
    assert!(invalid.stdout.is_empty());
    let error: serde_json::Value = serde_json::from_slice(&invalid.stderr).unwrap();
    assert!(
        error["errors"][0]["message"]
            .as_str()
            .unwrap()
            .contains("invalid JSON")
    );
}
