use cli::configuration::{EventConfig, EventSecrets, ServerConfig, read};
use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "animeitor-config-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
    fn write(&self, name: &str, text: &str) -> PathBuf {
        let p = self.0.join(name);
        std::fs::write(&p, text).unwrap();
        p
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
#[test]
fn active_manifests_are_self_contained_and_preserve_counts() {
    for (name, count) in [
        ("basic", 1),
        ("nacional_2026", 9),
        ("regional_2026", 10),
        ("latam_2026_2027", 2),
    ] {
        let e = EventConfig::load(&root().join(format!("config/{name}/event.toml"))).unwrap();
        assert_eq!(e.contests.len(), count);
        assert!(!e.event.secret.is_empty());
        assert!(
            e.configured()
                .iter()
                .all(|c| c.config.salt.is_some() && c.sites.iter().all(|s| s.salt.is_some()))
        );
    }
}
#[test]
fn private_sources_select_one_event_and_resolve_relative_to_file() {
    let t = Temp::new();
    let p = t.write(
        "private.toml",
        "[webcasts]\nfirst='inputs/a.zip'\nsecond='https://example.com/feed?key=private'\n",
    );
    assert_eq!(
        EventSecrets::source(&p, "first").unwrap(),
        t.0.join("inputs/a.zip").to_str().unwrap()
    );
    assert_eq!(
        EventSecrets::source(&p, "second").unwrap(),
        "https://example.com/feed?key=private"
    );
    assert!(EventSecrets::source(&p, "missing").is_err());
}
#[test]
fn invalid_private_toml_never_quotes_credentials() {
    let t = Temp::new();
    let p = t.write(
        "private.toml",
        "[webcasts]\nx = 'DO-NOT-PRINT-ME'\nx = 'other'\n",
    );
    let error = EventSecrets::source(&p, "x").unwrap_err().to_string();
    assert!(!error.contains("DO-NOT-PRINT-ME"));
    assert!(error.contains("private.toml"));
}
#[test]
fn rejects_unknown_fields_and_duplicate_names() {
    let t = Temp::new();
    let base = std::fs::read_to_string(root().join("config/basic/event.toml")).unwrap();
    let p = t.write("event.toml", &format!("unknown=true\n{base}"));
    assert!(EventConfig::load(&p).is_err());
    let p = t.write(
        "event.toml",
        &format!("{base}\n[[contests.sites]]\nname='Geral'\ncodes=['']\n"),
    );
    assert!(EventConfig::load(&p).is_err());
    let p = t.write(
        "event.toml",
        &base.replace("secret = \"development-event-value\"", "secret = \"\""),
    );
    assert!(EventConfig::load(&p).is_err());
    let p = t.write(
        "server.toml",
        &std::fs::read_to_string(root().join("server.toml.example")).unwrap(),
    );
    let mut s: ServerConfig = read(&p).unwrap();
    s.tokens
        .push(read::<ServerConfig>(&p).unwrap().tokens.remove(0));
    assert!(s.validate().is_err());
}
#[test]
fn generated_server_never_mounts_event_configuration_or_exposes_tokens() {
    let t = Temp::new();
    let output = t.0.join("generated");
    let server = root().join("server.docker.toml.example");
    let event = root().join("config/basic/event.toml");
    let private = root().join("event-secrets.toml.example");
    let value = cli::deployment::compose(&event, &private, &server, &output).unwrap();
    let json = serde_json::to_string(&value).unwrap();
    assert!(!json.contains("development-token"));
    assert!(!json.contains("development-server-salt"));
    let srv = serde_json::to_string(&value["services"]["animeitor"]).unwrap();
    assert!(!srv.contains("event.toml"));
    assert!(!srv.contains("event-secrets"));
    let print = serde_json::to_string(&value["services"]["printurls"]).unwrap();
    assert!(!print.contains("event-secrets"));
    assert!(!print.contains("depends_on"));
    assert_eq!(
        value["services"]["animeitor"]["command"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        std::fs::metadata(output.join("prometheus-token"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
}
#[test]
fn rejects_server_mount_exposing_event_files() {
    let t = Temp::new();
    let path = t.write(
        "server.toml",
        &format!(
            "{}\n[[docker.mounts]]\nsource = '{}'\ntarget = '/leak'\n",
            std::fs::read_to_string(root().join("server.toml.example")).unwrap(),
            root().display()
        ),
    );
    assert!(
        cli::deployment::compose(
            &root().join("config/basic/event.toml"),
            &root().join("event-secrets.toml.example"),
            &path,
            &t.0.join("out")
        )
        .is_err()
    );
}
