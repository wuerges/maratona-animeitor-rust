use super::*;
use axum::{
    Router,
    extract::{Request, State},
    http::StatusCode,
    response::IntoResponse,
    routing::any,
};
use clap::Parser;
use std::sync::{Arc, Mutex};

fn parse(words: &[&str]) -> RequestPlan {
    let args = args::AdminArgs::try_parse_from(
        std::iter::once("animeitor-admin").chain(words.iter().copied()),
    )
    .unwrap();
    plan(args.command).unwrap()
}
#[test]
fn cli_paths_and_payloads() {
    let cases = [
        (
            vec!["events", "list"],
            "GET",
            "/internal/events",
            Value::Null,
        ),
        (
            vec!["events", "get", "e"],
            "GET",
            "/internal/events/e",
            Value::Null,
        ),
        (
            vec!["contests", "get", "e", "c"],
            "GET",
            "/internal/contests/e/c",
            Value::Null,
        ),
        (
            vec!["sites", "get", "e", "c", "s"],
            "GET",
            "/internal/sites/e/c/s",
            Value::Null,
        ),
        (
            vec!["contests", "list", "e"],
            "GET",
            "/internal/events/e/contests",
            Value::Null,
        ),
        (
            vec!["sites", "list", "e", "c"],
            "GET",
            "/internal/events/e/contests/c/sites",
            Value::Null,
        ),
        (
            vec!["events", "update", "e", "--penalty-seconds", "600"],
            "PATCH",
            "/internal/events/e",
            json!({"penalty_seconds":600}),
        ),
        (
            vec![
                "contests", "update", "e", "c", "--gold", "4", "--unset", "style",
            ],
            "PATCH",
            "/internal/contests/e/c",
            json!({"ouro":4,"style":null}),
        ),
        (
            vec!["sites", "create", "e", "c", "s", "--code", "^t$"],
            "POST",
            "/internal/sites/e/c/s",
            json!({"name":"s","codes":["^t$"]}),
        ),
        (
            vec!["timer", "set", "e", "--seconds", "-3661"],
            "PATCH",
            "/internal/events/e/time",
            json!({"time_seconds":-3661}),
        ),
        (
            vec!["teams", "update", "e", "t", "--nome", "New"],
            "PATCH",
            "/internal/events/e/teams/t",
            json!({"nome":"New"}),
        ),
        (
            vec!["problems", "add", "e", "C"],
            "POST",
            "/internal/events/e/problems",
            json!({"problem":"C"}),
        ),
        (
            vec!["problems", "delete", "e", "C"],
            "DELETE",
            "/internal/events/e/problems/C",
            Value::Null,
        ),
        (
            vec!["events", "salt", "e"],
            "POST",
            "/internal/events/e/salt",
            json!({"salt":null}),
        ),
        (
            vec!["runs", "delete", "e", "-12"],
            "DELETE",
            "/internal/events/e/runs/-12",
            Value::Null,
        ),
        (
            vec!["runs", "clear", "e"],
            "DELETE",
            "/internal/events/e/runs",
            Value::Null,
        ),
        (
            vec!["revelation-urls", "e"],
            "GET",
            "/internal/events/e/revelation_urls",
            Value::Null,
        ),
    ];
    for (words, method, path, body) in cases {
        let request = parse(&words);
        assert_eq!(request.method.as_str(), method);
        assert_eq!(
            request_url("https://localhost/", &request).unwrap().path(),
            path
        );
        assert_eq!(request.body.unwrap_or(Value::Null), body);
    }
    let request = parse(&["teams", "delete", "regional 2026", "t/á&?", "--keep-runs"]);
    let url = request_url("https://localhost/base/", &request).unwrap();
    assert_eq!(
        url.path(),
        "/base/internal/events/regional%202026/teams/t%2F%C3%A1&%3F"
    );
    assert_eq!(url.query(), Some("keep_runs=true"));
    let request = parse(&[
        "runs",
        "add",
        "e",
        "--id",
        "1",
        "--team-login",
        "t",
        "--problem",
        "A",
        "--time-seconds",
        "56",
        "--answer",
        "Y",
    ]);
    assert_eq!(
        request.body.unwrap(),
        json!({"runs":[{"id":1,"team_login":"t","prob":"A","time_seconds":56,"answer":"Y"}]})
    );
}
#[test]
fn invalid_inputs_fail_locally() {
    for words in [
        vec!["events", "update", "e"],
        vec!["contests", "update", "e", "c", "--unset", "ouro"],
        vec![
            "contests", "update", "e", "c", "--style", "x", "--unset", "style",
        ],
        vec!["sites", "create", "e", "c", "s", "--code", "["],
        vec!["contests", "codes", "e", "c", "--add", "x", "--remove", "x"],
        vec!["contests", "codes", "e", "c"],
        vec![
            "events",
            "update",
            "e",
            "--time-seconds",
            "0",
            "--keep-runs",
        ],
        vec!["events", "get", ".."],
        vec!["teams", "add", "e", "--login", "t"],
    ] {
        let args = args::AdminArgs::try_parse_from(std::iter::once("admin").chain(words)).unwrap();
        assert!(plan(args.command).is_err());
    }
    assert!(
        args::AdminArgs::try_parse_from([
            "admin",
            "runs",
            "add",
            "e",
            "--id",
            "1",
            "--team-login",
            "t",
            "--problem",
            "A",
            "--time-seconds",
            "0",
            "--answer",
            "bad"
        ])
        .is_err()
    );
}
#[test]
fn file_inputs_and_defaults_are_validated() {
    let path =
        std::env::temp_dir().join(format!("animeitor-admin-input-{}.json", std::process::id()));
    std::fs::write(&path, r#"{"codes":[],"salt":null}"#).unwrap();
    let file = path.to_str().unwrap();
    let request = parse(&["sites", "create", "e", "c", "s", "--file", file]);
    assert_eq!(
        request.body.unwrap(),
        json!({"name":"s","codes":[],"salt":null})
    );
    for words in [
        vec![
            "sites", "update", "e", "c", "s", "--file", file, "--salt", "x",
        ],
        vec![
            "sites", "update", "e", "c", "s", "--file", file, "--unset", "salt",
        ],
    ] {
        let args = args::AdminArgs::try_parse_from(std::iter::once("admin").chain(words)).unwrap();
        assert!(plan(args.command).is_err());
    }
    for bad in [
        r#"{"time_seconds":null}"#,
        r#"{"typo":1}"#,
        r#"{"name":"other"}"#,
        r#"{"teams":[{"login":"t","escola":"S","nome":"T","typo":1}]}"#,
    ] {
        std::fs::write(&path, bad).unwrap();
        let args =
            args::AdminArgs::try_parse_from(["admin", "events", "update", "e", "--file", file])
                .unwrap();
        assert!(plan(args.command).is_err());
    }
    std::fs::write(
        &path,
        r#"{"problems":["A"],"teams":[],"score_freeze_time_seconds":100,"penalty_seconds":1200}"#,
    )
    .unwrap();
    let request = parse(&["events", "create", "e", "--file", file]);
    let event: data::event::EventState = serde_json::from_value(request.body.unwrap()).unwrap();
    assert_eq!(event.time_seconds, 0);
    std::fs::remove_file(path).unwrap();
}
#[test]
fn rendering_keeps_json_clean_and_warnings_visible() {
    let request = parse(&["events", "get", "e"]);
    let output = Output {
        status: 200,
        text: None,
        value: Some(
            json!({"data":{"added":1,"updated":0},"warnings":[{"code":"unknown_team","message":"skipped"}]}),
        ),
    };
    let (stdout, stderr) = render(&output, &request, true).unwrap();
    assert!(stderr.is_empty());
    assert!(serde_json::from_str::<Value>(&stdout).unwrap()["warnings"].is_array());
    let (stdout, stderr) = render(&output, &request, false).unwrap();
    assert!(stdout.contains("Added: 1"));
    assert!(stderr.contains("unknown_team"));
    let empty = Output {
        status: 204,
        text: None,
        value: None,
    };
    assert_eq!(render(&empty, &request, true).unwrap().0, "");
    let metrics = Output {
        status: 200,
        text: Some("sample 1\n".into()),
        value: None,
    };
    assert_eq!(render(&metrics, &request, false).unwrap().0, "sample 1\n");
    assert_eq!(
        serde_json::from_str::<Value>(&render(&metrics, &request, true).unwrap().0).unwrap(),
        json!({"data":"sample 1\n"})
    );
}

type Received = Arc<Mutex<Vec<(String, String, Value, String)>>>;
async fn capture(State(received): State<Received>, req: Request) -> axum::response::Response {
    let method = req.method().to_string();
    let path = req.uri().to_string();
    let auth = req
        .headers()
        .get("authorization")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let bytes = axum::body::to_bytes(req.into_body(), 1024 * 1024)
        .await
        .unwrap();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    received
        .lock()
        .unwrap()
        .push((method, path.clone(), body, auth));
    if path.ends_with("slow") {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    if path.ends_with("conflict") {
        return (
            StatusCode::CONFLICT,
            axum::Json(json!({"errors":[{"code":"conflict","message":"exists"}]})),
        )
            .into_response();
    }
    if path.ends_with("unauthorized") {
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({"errors":[{"code":"unauthorized","message":"invalid credentials"}]})),
        )
            .into_response();
    }
    if path.ends_with("missing") {
        return (
            StatusCode::NOT_FOUND,
            axum::Json(json!({"errors":[{"code":"not_found","message":"missing"}]})),
        )
            .into_response();
    }
    if path.ends_with("nonjson") {
        return (StatusCode::BAD_GATEWAY, "proxy error").into_response();
    }
    if path.ends_with("invalid") {
        return axum::Json(json!({"unexpected":true})).into_response();
    }
    if path.ends_with("empty") {
        return StatusCode::NO_CONTENT.into_response();
    }
    if path.ends_with("metrics") {
        return "metric 1\n".into_response();
    }
    axum::Json(json!({"data":{"teams":[],"problems":["A"]}})).into_response()
}
#[tokio::test]
async fn transport_sends_one_authenticated_request_and_handles_failures() {
    let received: Received = Default::default();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new()
        .fallback(any(capture))
        .with_state(received.clone());
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    // HTTP is used only for the isolated transport test. Production configuration requires HTTPS.
    let client = AdminClient {
        client: reqwest::Client::new(),
        base: format!("http://{address}"),
        username: "user".into(),
        token: "token".into(),
        timeout: Duration::from_millis(100),
    };
    let request = parse(&["teams", "delete", "e", "t", "--keep-runs"]);
    client.execute(&request).await.unwrap();
    assert_eq!(
        received.lock().unwrap()[0],
        (
            "DELETE".into(),
            "/internal/events/e/teams/t?keep_runs=true".into(),
            Value::Null,
            "Basic dXNlcjp0b2tlbg==".into()
        )
    );
    let request = parse(&["events", "update", "e", "--time-seconds", "-60"]);
    client.execute(&request).await.unwrap();
    assert_eq!(received.lock().unwrap()[1].2, json!({"time_seconds":-60}));
    assert_eq!(received.lock().unwrap().len(), 2); // PATCH never fetches or PUTs first.
    for (name, status, code) in [
        ("conflict", 409, "conflict"),
        ("unauthorized", 401, "unauthorized"),
        ("missing", 404, "not_found"),
        ("nonjson", 502, "http_error"),
        ("invalid", 200, "invalid_response"),
    ] {
        let before = received.lock().unwrap().len();
        let err = client
            .execute(&parse(&["events", "get", name]))
            .await
            .err()
            .unwrap();
        assert_eq!(err.status, Some(status));
        assert_eq!(err.envelope["errors"][0]["code"], code);
        assert_eq!(received.lock().unwrap().len(), before + 1);
    }
    assert!(
        client
            .execute(&parse(&["events", "delete", "empty"]))
            .await
            .unwrap()
            .value
            .is_none()
    );
    assert_eq!(
        client
            .execute(&parse(&["problems", "list", "e"]))
            .await
            .unwrap()
            .value
            .unwrap(),
        json!({"data":["A"]})
    );
    assert_eq!(
        client
            .execute(&parse(&["metrics"]))
            .await
            .unwrap()
            .text
            .unwrap(),
        "metric 1\n"
    );
    assert_eq!(
        client
            .execute(&parse(&["events", "get", "slow"]))
            .await
            .err()
            .unwrap()
            .envelope["errors"][0]["code"],
        "timeout"
    );
    task.abort();
}

#[test]
fn server_url_override_is_global_and_optional() {
    for words in [
        vec![
            "animeitor-admin",
            "--server-url",
            "https://example.com/prefix",
            "events",
            "list",
        ],
        vec![
            "animeitor-admin",
            "events",
            "list",
            "--server-url",
            "https://example.com/prefix",
        ],
    ] {
        let args = args::AdminArgs::try_parse_from(words).unwrap();
        assert_eq!(
            args.server_url.as_deref(),
            Some("https://example.com/prefix")
        );
        let request = plan(args.command).unwrap();
        assert_eq!(
            request_url(args.server_url.as_deref().unwrap(), &request)
                .unwrap()
                .as_str(),
            "https://example.com/prefix/internal/events"
        );
    }
    let args = args::AdminArgs::try_parse_from(["animeitor-admin", "events", "list"]).unwrap();
    assert!(args.server_url.is_none());
}

#[test]
fn token_override_is_global_optional_and_nonempty() {
    for words in [
        vec![
            "animeitor-admin",
            "--token",
            "override-token",
            "events",
            "list",
        ],
        vec![
            "animeitor-admin",
            "events",
            "list",
            "--token",
            "override-token",
        ],
    ] {
        let args = args::AdminArgs::try_parse_from(words).unwrap();
        assert_eq!(args.token.as_deref(), Some("override-token"));
    }
    let args = args::AdminArgs::try_parse_from(["animeitor-admin", "events", "list"]).unwrap();
    assert!(args.token.is_none());
    assert!(
        args::AdminArgs::try_parse_from(["animeitor-admin", "--token", "", "events", "list"])
            .is_err()
    );
}

#[test]
fn media_flags_update_the_event() {
    let request = parse(&[
        "events",
        "update",
        "e",
        "--photo-url-format",
        "https://example.com/{team_login}",
        "--unset",
        "sound_url_format",
    ]);
    assert_eq!(request.segments, ["internal", "events", "e"]);
    assert_eq!(
        request.body,
        Some(json!({
            "photo_url_format":"https://example.com/{team_login}", "sound_url_format":null
        }))
    );
    assert!(
        args::AdminArgs::try_parse_from([
            "animeitor-admin",
            "contests",
            "update",
            "e",
            "c",
            "--photo-url-format",
            "https://example.com/{team_login}"
        ])
        .is_err()
    );
}
