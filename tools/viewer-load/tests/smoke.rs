//! Small CI workload exercises real API replay, corrections, and reconnects.
use serde_json::json;
use service::event_store::EventStore;
use std::{sync::Arc, time::Duration};
#[tokio::test]
async fn workload_reconnects_and_validates_corrected_state() {
    let store = EventStore::new(
        Arc::new(database_memory::MemoryDatabase::new()),
        "salt".into(),
    );
    store.create_event("load", serde_json::from_value(json!({"name":"load","problems":["A"],"teams":[{"login":"team1","nome":"Team","escola":"School"}],"score_freeze_time_seconds":240,"penalty_seconds":1200,"time_seconds":60})).unwrap()).await.unwrap();
    store
        .create_contest(
            "load",
            "all",
            serde_json::from_value(json!({"name":"all","codes":["team"]})).unwrap(),
        )
        .await
        .unwrap();
    let run = |answer: &str| {
        serde_json::from_value::<data::event::Run>(
            json!({"id":1,"team_login":"team1","prob":"A","time_seconds":10,"answer":answer}),
        )
        .unwrap()
    };
    store.add_runs("load", vec![run("Y")]).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = server_v2::app(server_v2::AppState {
        store: store.clone(),
        public_url: format!("http://{addr}").parse().unwrap(),
        internal_tokens: Default::default(),
    });
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let directory = tempfile::tempdir().unwrap();
    let expected = directory.path().join("expected.json");
    let report = directory.path().join("report.json");
    let trace = directory.path().join("trace.jsonl");
    std::fs::write(
        &expected,
        serde_json::to_vec(
            &json!({"runs":[run("Y")],"timer":store.current_timer("load").await.unwrap().unwrap()}),
        )
        .unwrap(),
    )
    .unwrap();
    let mut child = tokio::process::Command::new(env!("CARGO_BIN_EXE_viewer-load"))
        .args([
            "--target",
            &format!("http://{addr}"),
            "--event",
            "load",
            "--contest",
            "all",
            "--viewers",
            "3",
            "--ramp-seconds",
            "1",
            "--hold-seconds",
            "4",
            "--reconnect",
            "all",
            "--expect-file",
        ])
        .arg(&expected)
        .arg("--output")
        .arg(&report)
        .arg("--trace")
        .arg(&trace)
        .spawn()
        .unwrap();
    tokio::time::sleep(Duration::from_millis(1500)).await;
    store.add_runs("load", vec![run("N")]).await.unwrap();
    let next = directory.path().join("next.json");
    std::fs::write(
        &next,
        serde_json::to_vec(
            &json!({"runs":[run("N")],"timer":store.current_timer("load").await.unwrap().unwrap()}),
        )
        .unwrap(),
    )
    .unwrap();
    std::fs::rename(next, &expected).unwrap();
    assert!(
        tokio::time::timeout(Duration::from_secs(15), child.wait())
            .await
            .unwrap()
            .unwrap()
            .success()
    );
    let result: serde_json::Value =
        serde_json::from_slice(&std::fs::read(report).unwrap()).unwrap();
    assert_eq!(result["peak_sockets"], 6);
    assert!(result["opened_sockets"].as_u64().unwrap() >= 12);
    assert_eq!(result["validated_streams"], 6);
    assert_eq!(result["final_mismatches"], 0);
    assert_eq!(result["completed_initial_replays"], 3);
    assert!(
        std::fs::read_to_string(trace)
            .unwrap()
            .contains("received_unix_ms")
    );
    assert_eq!(result["unexpected_disconnects"], 0);
    assert_eq!(result["trace_dropped"], 0);
    assert!(result["samples"].as_array().unwrap().len() >= 3);
    server.abort();
}

#[tokio::test]
async fn publisher_seeds_updates_corrects_and_freezes_over_verified_https() {
    let directory = tempfile::tempdir().unwrap();
    let cert = directory.path().join("cert.pem");
    let key = directory.path().join("key.pem");
    assert!(
        std::process::Command::new("openssl")
            .args([
                "req",
                "-x509",
                "-newkey",
                "rsa:2048",
                "-nodes",
                "-days",
                "1",
                "-subj",
                "/CN=localhost",
                "-addext",
                "subjectAltName=DNS:localhost"
            ])
            .arg("-keyout")
            .arg(&key)
            .arg("-out")
            .arg(&cert)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .unwrap()
            .success()
    );
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let addr = listener.local_addr().unwrap();
    let store = EventStore::new(
        Arc::new(database_memory::MemoryDatabase::new()),
        "salt".into(),
    );
    let token = service::internal_auth::InternalToken::new(
        "load".into(),
        "test-only".into(),
        data::internal_auth::InternalRole::ReadWrite,
        vec!["capacity-test".into()],
    )
    .unwrap();
    let app = server_v2::app(server_v2::AppState {
        store: store.clone(),
        public_url: format!("https://localhost:{}", addr.port())
            .parse()
            .unwrap(),
        internal_tokens: Arc::new(std::collections::HashMap::from([("load".into(), token)])),
    });
    let tls = axum_server::tls_rustls::RustlsConfig::from_pem_file(&cert, &key)
        .await
        .unwrap();
    let server = tokio::spawn(async move {
        axum_server::from_tcp_rustls(listener, tls)
            .unwrap()
            .serve(app.into_make_service())
            .await
            .unwrap();
    });
    let config = directory.path().join("server.toml");
    std::fs::write(&config,format!("server_url='https://localhost:{}'\ntls_ca_cert='cert.pem'\nclient_token='load'\n[[tokens]]\nname='load'\ntoken='test-only'\n",addr.port())).unwrap();
    let expected = directory.path().join("expected.json");
    let script =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/capacity_workload.py");
    for mode in ["seed", "live"] {
        let status = tokio::process::Command::new("python3")
            .arg(&script)
            .arg(mode)
            .arg("--server-config")
            .arg(&config)
            .arg("--expected")
            .arg(&expected)
            .arg("--ledger")
            .arg(directory.path().join("ledger.jsonl"))
            .args([
                "--teams",
                "2",
                "--runs",
                "20",
                "--seconds",
                "3",
                "--correct-at",
                "1",
                "--freeze-at",
                "2",
            ])
            .status()
            .await
            .unwrap();
        assert!(status.success(), "publisher {mode} failed");
    }
    let expected: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&expected).unwrap()).unwrap();
    assert_eq!(expected["runs"].as_array().unwrap().len(), 50);
    assert!(
        expected["runs"]
            .as_array()
            .unwrap()
            .iter()
            .all(|run| run["answer"] == "?")
    );
    assert_eq!(expected["timer"]["current_time_seconds"], 63);
    assert_eq!(expected["timer"]["score_freeze_time_seconds"], 0);
    server.abort();
}
