use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use server_v2::{AppState, app};
use service::event_store::EventStore;
use tower::ServiceExt;

async fn send(app: &Router, method: &str, path: &str, body: Value) -> (StatusCode, Value) {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header(
                    "Authorization",
                    format!("Basic {}", STANDARD.encode("operator:token")),
                )
                .header("Content-Type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap()
        },
    )
}
async fn setup() -> (Router, EventStore) {
    let store = test_store(Some("server-secret".into()));
    let app = app(AppState {
        store: store.clone(),
        public_url: "https://example.com".parse().unwrap(),
        internal_tokens: std::sync::Arc::new(std::collections::HashMap::from([(
            "operator".into(),
            "token".into(),
        )])),
    });
    for (path, body) in [
        (
            "/internal/events/e",
            json!({"name":"e","problems":["A","B"],"teams":[{"login":"t1","escola":"School","nome":"One"}],"score_freeze_time_seconds":100,"penalty_seconds":1200,"time_seconds":-60,"salt":"old"}),
        ),
        (
            "/internal/contests/e/c",
            json!({"name":"c","codes":["^t1$"],"salt":"old"}),
        ),
        (
            "/internal/sites/e/c/s",
            json!({"name":"s","codes":["^t1$"],"salt":"old"}),
        ),
    ] {
        assert_eq!(send(&app, "POST", path, body).await.0, 201);
    }
    (app, store)
}
#[tokio::test]
async fn patches_preserve_fields_and_validate_before_mutation() {
    let (app, store) = setup().await;
    for (path, patch) in [
        (
            "/internal/events/e",
            json!({"penalty_seconds":600,"salt":null}),
        ),
        ("/internal/contests/e/c", json!({"ouro":4,"salt":null})),
        ("/internal/sites/e/c/s", json!({"salt":null})),
    ] {
        let (status, value) = send(&app, "PATCH", path, patch).await;
        assert_eq!(status, 200);
        assert_eq!(value["data"]["salt"], Value::Null);
        assert_eq!(send(&app, "GET", path, Value::Null).await.1, value);
    }
    let event = store.get_event("e").await.unwrap().unwrap();
    assert_eq!(event.time_seconds, -60);
    assert_eq!(event.teams.len(), 1);
    for body in [
        json!({}),
        json!({"time_seconds":null}),
        json!({"name":"renamed"}),
        json!({"typo":1}),
    ] {
        assert_eq!(send(&app, "PATCH", "/internal/events/e", body).await.0, 400);
        assert_eq!(store.get_event("e").await.unwrap().unwrap(), event);
    }
    let contest = store.get_contest("e", "c").await.unwrap().unwrap();
    assert_eq!(
        send(
            &app,
            "PATCH",
            "/internal/contests/e/c",
            json!({"ouro":99,"codes":["["]})
        )
        .await
        .0,
        400
    );
    assert_eq!(store.get_contest("e", "c").await.unwrap().unwrap(), contest);
    assert!(
        store
            .contest_codes("e", "c")
            .await
            .unwrap()
            .unwrap()
            .is_match("t1")
    );
    for path in [
        "/internal/events/missing",
        "/internal/contests/e/missing",
        "/internal/sites/e/c/missing",
    ] {
        assert_eq!(send(&app, "PATCH", path, json!({"salt":null})).await.0, 404);
    }
}
#[tokio::test]
async fn referenced_removals_are_atomic_and_keep_runs_is_explicit() {
    let (app, store) = setup().await;
    let run = json!({"runs":[{"id":1,"team_login":"t1","prob":"A","time_seconds":1,"answer":"Y"}]});
    assert_eq!(
        send(&app, "POST", "/internal/events/e/runs", run.clone())
            .await
            .0,
        200
    );
    for path in [
        "/internal/events/e/teams/t1",
        "/internal/events/e/problems/A",
    ] {
        assert_eq!(send(&app, "DELETE", path, Value::Null).await.0, 409);
    }
    let before = store.get_event("e").await.unwrap().unwrap();
    assert_eq!(
        send(
            &app,
            "PATCH",
            "/internal/events/e",
            json!({"teams":[],"penalty_seconds":1})
        )
        .await
        .0,
        409
    );
    assert_eq!(
        send(
            &app,
            "PATCH",
            "/internal/events/e?keep_runs=true",
            json!({"teams":[],"problems":[]})
        )
        .await
        .0,
        409
    );
    assert_eq!(store.get_event("e").await.unwrap().unwrap(), before);
    assert_eq!(
        send(
            &app,
            "DELETE",
            "/internal/events/e/teams/t1?keep_runs=true",
            Value::Null
        )
        .await
        .0,
        204
    );
    assert_eq!(
        store.site_runs("e", "c", "s").await.unwrap().unwrap().len(),
        1
    );
    assert_eq!(
        send(&app, "POST", "/internal/events/e/runs", run).await.1["warnings"][0]["code"],
        "unknown_team"
    );
    let body = json!({"login":"t1","escola":"School","nome":"Restored"});
    assert_eq!(
        send(&app, "POST", "/internal/events/e/teams", body).await.0,
        201
    );
    assert_eq!(
        store.site_runs("e", "c", "s").await.unwrap().unwrap().len(),
        1
    );
    assert_eq!(
        send(
            &app,
            "PATCH",
            "/internal/events/e?keep_runs=true",
            json!({"teams":[]})
        )
        .await
        .0,
        200
    );
    assert_eq!(
        store.site_runs("e", "c", "s").await.unwrap().unwrap().len(),
        1
    );
}
#[tokio::test]
async fn collection_operations_preserve_order_and_filter_compilation() {
    let (app, store) = setup().await;
    let team = json!({"login":"t2","escola":"S","nome":"Two"});
    assert_eq!(
        send(&app, "POST", "/internal/events/e/teams", team.clone())
            .await
            .0,
        201
    );
    assert_eq!(
        send(&app, "POST", "/internal/events/e/teams", team).await.0,
        409
    );
    assert_eq!(
        send(
            &app,
            "PATCH",
            "/internal/events/e/teams/t2",
            json!({"nome":"Second"})
        )
        .await
        .1["data"]["escola"],
        "S"
    );
    assert_eq!(
        send(&app, "GET", "/internal/events/e/teams/t2", Value::Null)
            .await
            .1["data"]["nome"],
        "Second"
    );
    assert_eq!(
        send(
            &app,
            "PATCH",
            "/internal/events/e/teams/t2",
            json!({"login":"x"})
        )
        .await
        .0,
        400
    );
    assert_eq!(
        send(&app, "DELETE", "/internal/events/e/teams/t2", Value::Null)
            .await
            .0,
        204
    );
    assert_eq!(
        send(&app, "DELETE", "/internal/events/e/teams/t2", Value::Null)
            .await
            .0,
        404
    );
    assert_eq!(
        send(
            &app,
            "POST",
            "/internal/events/e/problems",
            json!({"problem":"C"})
        )
        .await
        .1["data"],
        json!(["A", "B", "C"])
    );
    assert_eq!(
        send(
            &app,
            "POST",
            "/internal/events/e/problems",
            json!({"problem":"C"})
        )
        .await
        .0,
        409
    );
    assert_eq!(
        send(&app, "DELETE", "/internal/events/e/problems/B", Value::Null)
            .await
            .0,
        204
    );
    for path in [
        "/internal/contests/e/c/codes",
        "/internal/sites/e/c/s/codes",
    ] {
        let delta = json!({"add":["^t2$","^t3$"],"remove":["^t1$"]});
        let result = send(&app, "PATCH", path, delta.clone()).await;
        assert_eq!(result.0, 200);
        assert_eq!(result.1["data"]["codes"], json!(["^t2$", "^t3$"]));
        assert_eq!(send(&app, "PATCH", path, delta).await, result);
        for bad in [
            json!({}),
            json!({"add":["["]}),
            json!({"add":["x"],"remove":["x"]}),
        ] {
            assert_eq!(send(&app, "PATCH", path, bad).await.0, 400);
        }
    }
    let codes = store.contest_codes("e", "c").await.unwrap().unwrap();
    assert!(codes.is_match("t2"));
    assert!(!codes.is_match("t1"));
}
#[tokio::test]
async fn concurrent_patches_merge_under_the_store_lock() {
    let (app, store) = setup().await;
    let mut timer = store.subscribe_timer("e").await.unwrap().unwrap();
    let (a, b, c) = tokio::join!(
        send(
            &app,
            "PATCH",
            "/internal/events/e",
            json!({"penalty_seconds":600})
        ),
        send(
            &app,
            "PATCH",
            "/internal/events/e",
            json!({"time_seconds":25})
        ),
        send(
            &app,
            "POST",
            "/internal/events/e/teams",
            json!({"login":"t2","escola":"S","nome":"Two"})
        )
    );
    assert_eq!((a.0.as_u16(), b.0.as_u16(), c.0.as_u16()), (200, 200, 201));
    let event = store.get_event("e").await.unwrap().unwrap();
    assert_eq!(event.time_seconds, 25);
    assert_eq!(event.penalty_seconds, 600);
    assert_eq!(event.teams.len(), 2);
    assert_eq!(timer.recv().await.unwrap().current_time_seconds, 25);
}
#[tokio::test]
async fn legacy_duplicates_and_authentication() {
    let (app, store) = setup().await;
    let mut event = store.get_event("e").await.unwrap().unwrap();
    event.teams.push(event.teams[0].clone());
    event.problems.push("A".into());
    store.put_event("e", event).await.unwrap();
    for (method, path, body) in [
        ("PATCH", "/internal/events/e/teams/t1", json!({"nome":"x"})),
        ("DELETE", "/internal/events/e/problems/A", Value::Null),
    ] {
        assert_eq!(send(&app, method, path, body).await.0, 409);
    }
    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/internal/events/e")
                .header("Content-Type", "application/json")
                .body(Body::from("{\"time_seconds\":0}"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn admin_command_sequence_uses_real_routes() {
    use clap::Parser;
    use cli::admin::{args::AdminArgs, plan, request_url};
    let (app, _) = setup().await;
    async fn command(app: &Router, words: &[&str]) -> (StatusCode, Value) {
        let args = AdminArgs::try_parse_from(std::iter::once("admin").chain(words.iter().copied()))
            .unwrap();
        let request = plan(args.command).unwrap();
        let url = request_url("https://example.com", &request).unwrap();
        let path = format!(
            "{}{}",
            url.path(),
            url.query().map(|q| format!("?{q}")).unwrap_or_default()
        );
        send(
            app,
            request.method.as_str(),
            &path,
            request.body.unwrap_or(Value::Null),
        )
        .await
    }
    let steps = [
        (
            vec!["contests", "create", "e", "regional", "--code", "^t"],
            201,
        ),
        (
            vec!["sites", "create", "e", "regional", "campus", "--code", "^t"],
            201,
        ),
        (
            vec![
                "teams", "add", "e", "--login", "t2", "--escola", "School", "--nome", "Two",
            ],
            201,
        ),
        (vec!["problems", "add", "e", "C"], 201),
        (
            vec!["contests", "update", "e", "regional", "--gold", "4"],
            200,
        ),
        (
            vec![
                "sites", "codes", "e", "regional", "campus", "--add", "^other$",
            ],
            200,
        ),
        (
            vec![
                "runs",
                "add",
                "e",
                "--id",
                "1",
                "--team-login",
                "t2",
                "--problem",
                "C",
                "--time-seconds",
                "56",
                "--answer",
                "Y",
            ],
            200,
        ),
        (vec!["teams", "delete", "e", "t2"], 409),
        (vec!["teams", "delete", "e", "t2", "--keep-runs"], 204),
        (vec!["timer", "set", "e", "--seconds", "-120"], 200),
        (vec!["revelation-urls", "e"], 200),
        (vec!["runs", "clear", "e"], 204),
        (vec!["problems", "delete", "e", "C"], 204),
        (vec!["sites", "delete", "e", "regional", "campus"], 204),
        (vec!["contests", "delete", "e", "regional"], 204),
    ];
    for (words, status) in steps {
        assert_eq!(command(&app, &words).await.0.as_u16(), status, "{words:?}");
    }
}

fn test_store(salt: Option<String>) -> service::event_store::EventStore {
    service::event_store::EventStore::new(
        std::sync::Arc::new(database_memory::MemoryDatabase::new()),
        salt.unwrap_or_else(|| "test-server-salt".into()),
    )
}
