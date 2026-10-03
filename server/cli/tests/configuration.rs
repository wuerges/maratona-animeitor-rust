use cli::configuration::{EventConfig, EventSecrets, ServerConfig};
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
        ("jones", 1),
        ("colombia", 2),
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
    let error = EventSecrets::source(&p, "colombia")
        .unwrap_err()
        .to_string();
    assert!(error.contains(&p.display().to_string()));
    assert!(error.contains("event.name"));
    assert!(
        error.contains("[webcasts]\n\"colombia\" = \"https://example.com/private/webcast.zip\"")
    );
    assert!(error.contains("Relative paths"));
    assert!(error.contains("feeder container"));
    assert!(!error.contains("key=private"));
}
#[test]
fn missing_webcasts_table_has_setup_instructions() {
    let t = Temp::new();
    let path = t.write("event-secrets.toml", "# Not configured yet\n");
    let error = EventSecrets::source(&path, "colombia").unwrap_err();
    assert!(error.is::<cli::configuration::MissingWebcast>());
    assert!(error.to_string().contains("[webcasts]"));
    assert!(error.to_string().contains("\"colombia\" ="));
}

#[test]
fn feeder_prints_missing_webcast_once() {
    let t = Temp::new();
    let secrets = t.write("event-secrets.toml", "[webcasts]\n");
    for (filter, expected_count) in [("info", 1), ("off", 1)] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_animeitor-feeder"))
            .args(["--event-config"])
            .arg(root().join("config/jones/event.toml"))
            .arg("--server-config")
            .arg(root().join("server.docker.toml.example"))
            .arg("--event-secrets")
            .arg(&secrets)
            .env("RUST_LOG", filter)
            .env_remove("SENTRY_DSN")
            .output()
            .unwrap();
        assert!(!output.status.success());
        let console = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            console.matches("has no webcast for selected event").count(),
            expected_count,
            "{console}"
        );
        assert_eq!(console.contains("[webcasts]"), expected_count == 1);
        assert!(String::from_utf8_lossy(&output.stderr).contains("Error: "));
        assert!(!String::from_utf8_lossy(&output.stdout).contains("has no webcast"));
    }
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
    let mut s = ServerConfig::load(&p).unwrap();
    s.tokens
        .push(ServerConfig::load(&p).unwrap().tokens.remove(0));
    assert!(s.validate().is_err());
}

#[tokio::test]
async fn jones_fixture_populates_the_public_scoreboard() {
    use service::event_store::from_legacy_contest_state;
    let config = EventConfig::load(&root().join("config/jones/event.toml")).unwrap();
    let source = EventSecrets::source(
        &root().join("event-secrets.toml.example"),
        &config.event.name,
    )
    .unwrap();
    let legacy = service::webcast::load_data_from_url_maybe(&source)
        .await
        .unwrap();
    let (mut event, runs) = from_legacy_contest_state(&legacy, &config.event.name);
    event.salt = Some(config.event.secret.clone());
    let store = test_store(None);
    store.create_event("jones", event).await.unwrap();
    for contest in config.configured() {
        store
            .create_contest("jones", &contest.config.name.clone(), contest.config)
            .await
            .unwrap();
        for site in contest.sites {
            store
                .create_site("jones", "Jones", &site.name.clone(), site)
                .await
                .unwrap();
        }
    }
    store.add_runs("jones", runs).await.unwrap();
    let public = store.public_state("jones", "Jones").await.unwrap().unwrap();
    assert_eq!(public.teams.len(), 20);
    assert_eq!(public.problems.unwrap().len(), 8);
    assert_eq!(
        store
            .contest_runs("jones", "Jones")
            .await
            .unwrap()
            .unwrap()
            .len(),
        134
    );
    assert_eq!(
        store
            .site_runs("jones", "Jones", "Geral")
            .await
            .unwrap()
            .unwrap()
            .len(),
        134
    );
}

fn test_store(salt: Option<String>) -> service::event_store::EventStore {
    service::event_store::EventStore::new(
        std::sync::Arc::new(database_memory::MemoryDatabase::new()),
        salt.unwrap_or_else(|| "test-server-salt".into()),
    )
}

#[test]
fn database_defaults_and_sqlite_paths_are_relative_to_server_config() {
    use service::database::DatabaseConfig;
    let temp = Temp::new();
    let base = std::fs::read_to_string(root().join("server.toml.example")).unwrap();
    let path = temp.write("server.toml", &base);
    assert!(matches!(
        ServerConfig::load(&path).unwrap().database,
        DatabaseConfig::Memory {}
    ));
    let path = temp.write(
        "server.toml",
        &format!("{base}\n[database]\ntype='sqlite'\npath='var/contest.sqlite3'\n"),
    );
    let DatabaseConfig::Sqlite { path } = ServerConfig::load(&path).unwrap().database else {
        panic!("expected SQLite")
    };
    assert_eq!(path, temp.0.join("var/contest.sqlite3"));
    // Parsing administrative configuration does not create or open a database.
    assert!(!path.exists());
    for section in [
        "type='unknown'",
        "type='sqlite'",
        "type='sqlite'\npath=''",
        "type='memory'\npath='unexpected'",
        "type='sqlite'\npath=':memory:'",
    ] {
        let path = temp.write("server.toml", &format!("{base}\n[database]\n{section}\n"));
        assert!(ServerConfig::load(&path).is_err(), "{section}");
    }
}

#[test]
fn inline_credentials_select_enabled_tokens_and_compile_permissions() {
    let temp = Temp::new();
    let server = std::fs::read_to_string(root().join("server.toml.example")).unwrap();
    let path = temp.write("server.toml", &server);
    let config = ServerConfig::load(&path).unwrap();
    let credential = config.credential().unwrap();
    assert_eq!(credential.name, "feeder");
    assert!(credential.owns("regional-future"));
    assert!(!credential.owns("old-regional-future"));
    assert!(!temp.0.join("internal_tokens.toml").exists());
    let path = temp.write(
        "server.toml",
        &server.replace("client_token = \"feeder\"", "client_token = \"observer\""),
    );
    let config = ServerConfig::load(&path).unwrap();
    assert_eq!(
        config.credential().unwrap().role,
        data::internal_auth::InternalRole::ReadOnly
    );
    let path = temp.write(
        "server.toml",
        &server.replace("enabled = true", "enabled = false"),
    );
    assert!(ServerConfig::load(&path).is_err());
}

#[test]
fn invalid_inline_permissions_are_rejected_without_exposing_secrets() {
    let temp = Temp::new();
    let server = std::fs::read_to_string(root().join("server.toml.example")).unwrap();
    let start = server.find("[[tokens]]").unwrap();
    let end = server.find("[[assets]]").unwrap();
    let base = format!("{}{}", &server[..start], &server[end..]);
    let valid = "[[tokens]]\nname='feeder'\ntoken='DO-NOT-PRINT-ME'\nrole='read-write'\nevents=['contest-.*']\n";
    for invalid in [
        valid.replace("role='read-write'\n", ""),
        valid.replace("events=['contest-.*']\n", ""),
        valid.replace("read-write", "admin"),
        valid.replace("contest-.*", "["),
        valid.replace("contest-.*", "   "),
        valid.replace("['contest-.*']", "['contest-.*', 'contest-.*']"),
        valid.replace("name='feeder'", "name='bad:name'"),
        format!("{valid}\n{valid}"),
        format!("{valid}unknown='DO-NOT-PRINT-ME'\n"),
        valid.replace("role='read-write'", "enabled=false"),
    ] {
        let path = temp.write("server.toml", &format!("{base}\n{invalid}"));
        let error = ServerConfig::load(&path).err().unwrap().to_string();
        assert!(!error.contains("DO-NOT-PRINT-ME"));
    }
    let path = temp.write("server.toml", &format!("{base}\n{valid}"));
    assert!(ServerConfig::load(&path).is_ok());
}
