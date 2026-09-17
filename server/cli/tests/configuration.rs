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
        ("jones", 1),
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

#[tokio::test]
async fn jones_fixture_populates_the_public_scoreboard() {
    use service::event_store::{EventStore, from_legacy_contest_state};
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
    let store = EventStore::new();
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
    let public = store.public_state("jones", "Jones").await.unwrap();
    assert_eq!(public.teams.len(), 20);
    assert_eq!(public.problems.unwrap().len(), 8);
    assert_eq!(
        store.contest_runs("jones", "Jones").await.unwrap().len(),
        134
    );
    assert_eq!(
        store
            .site_runs("jones", "Jones", "Geral")
            .await
            .unwrap()
            .len(),
        134
    );
}
