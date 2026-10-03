use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use data::internal_auth::InternalRole;
use http_body_util::BodyExt;
use serde_json::{Value, json};
use service::internal_auth::InternalToken;
use tower::ServiceExt;
use utoipa::OpenApi;

mod common;

fn token(name: &str, role: InternalRole, patterns: &[&str]) -> InternalToken {
    InternalToken::new(
        name.into(),
        "private-secret".into(),
        role,
        patterns.iter().map(|p| (*p).into()).collect(),
    )
    .unwrap()
}
async fn fixture() -> (axum::Router, service::event_store::EventStore) {
    let store = common::test_store(None);
    for name in ["contest-one", "other", "contest-two", "old-contest-one"] {
        store
            .create_event(
                name,
                serde_json::from_value(json!({
                    "name": name, "problems": ["A"], "teams": [],
                    "score_freeze_time_seconds": 100, "penalty_seconds": 1200, "time_seconds": -60
                }))
                .unwrap(),
            )
            .await
            .unwrap();
    }
    let mut disabled = token("disabled", InternalRole::ReadWrite, &[".*"]);
    disabled.enabled = false;
    let tokens = [
        token("reader", InternalRole::ReadOnly, &["contest-.*"]),
        token("writer", InternalRole::ReadWrite, &["contest-.*"]),
        token("exact", InternalRole::ReadOnly, &["contest-one", "other"]),
        token("empty", InternalRole::ReadOnly, &[]),
        disabled,
    ]
    .into_iter()
    .map(|t| (t.name.clone(), t))
    .collect();
    (
        server_v2::app(server_v2::AppState {
            store: store.clone(),
            public_url: "https://example.com".parse().unwrap(),
            internal_tokens: std::sync::Arc::new(tokens),
        }),
        store,
    )
}
async fn request(
    app: &axum::Router,
    user: Option<&str>,
    method: &str,
    path: &str,
    body: &str,
) -> (StatusCode, axum::http::HeaderMap, Value) {
    let mut req = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json");
    if let Some(user) = user {
        req = req.header(
            "authorization",
            format!(
                "Basic {}",
                STANDARD.encode(format!("{user}:private-secret"))
            ),
        );
    }
    let response = app
        .clone()
        .oneshot(req.body(Body::from(body.to_owned())).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        headers,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

#[tokio::test]
async fn capabilities_and_global_reads_do_not_require_events() {
    let (app, _) = fixture().await;
    for (user, role, patterns) in [
        ("reader", "read-only", json!(["contest-.*"])),
        ("writer", "read-write", json!(["contest-.*"])),
        ("empty", "read-only", json!([])),
    ] {
        let (status, headers, body) =
            request(&app, Some(user), "GET", "/internal/capabilities", "").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers["cache-control"], "no-store");
        assert_eq!(
            body,
            json!({"data": {"name": user, "role": role, "events": patterns}})
        );
        assert!(!body.to_string().contains("private-secret"));
        for path in [
            "/internal/docs",
            "/internal/openapi.json",
            "/internal/metrics",
        ] {
            let (status, _, _) = request(&app, Some(user), "GET", path, "").await;
            assert_eq!(status, StatusCode::OK, "{path}");
        }
    }
    for user in [None, Some("unknown"), Some("disabled")] {
        let (status, headers, body) =
            request(&app, user, "GET", "/internal/capabilities", "").await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(headers["cache-control"], "no-store");
        assert_eq!(headers["www-authenticate"], "Basic");
        assert_eq!(body["errors"][0]["code"], "unauthorized");
    }
}

#[tokio::test]
async fn ownership_filters_lists_and_matches_whole_decoded_names() {
    let (app, _) = fixture().await;
    for (user, names) in [
        ("reader", json!(["contest-one", "contest-two"])),
        ("writer", json!(["contest-one", "contest-two"])),
        ("exact", json!(["contest-one", "other"])),
        ("empty", json!([])),
    ] {
        let (status, _, body) = request(&app, Some(user), "GET", "/internal/events", "").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["data"], names);
    }
    assert_eq!(
        request(
            &app,
            Some("reader"),
            "GET",
            "/internal/events/contest%2Done",
            ""
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &app,
            Some("reader"),
            "HEAD",
            "/internal/events/contest-one",
            ""
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        request(
            &app,
            Some("exact"),
            "GET",
            "/internal/events/old-contest-one",
            ""
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    // Ownership is checked before existence, including inaccessible future names.
    assert_eq!(
        request(
            &app,
            Some("reader"),
            "GET",
            "/internal/events/private-future",
            ""
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn every_documented_event_route_enforces_role_and_ownership_before_body_parsing() {
    let (app, store) = fixture().await;
    let before = serde_json::to_value(store.get_event("contest-one").await.unwrap()).unwrap();
    let spec = serde_json::to_value(server_v2::openapi::InternalApiDoc::openapi()).unwrap();
    let mut writes = 0;
    for (template, methods) in spec["paths"].as_object().unwrap() {
        if !template.contains("{event_name}") {
            continue;
        }
        for method in methods.as_object().unwrap().keys() {
            if !["get", "post", "put", "patch", "delete"].contains(&method.as_str()) {
                continue;
            }
            let path = template
                .replace("{event_name}", "contest-one")
                .replace("{contest_name}", "c")
                .replace("{site_name}", "s")
                .replace("{run_id}", "1")
                .replace("{login}", "team")
                .replace("{problem}", "A");
            let method = method.to_uppercase();
            if method != "GET" {
                writes += 1;
                let (status, _, body) =
                    request(&app, Some("reader"), &method, &path, "invalid-json").await;
                assert_eq!(status, StatusCode::FORBIDDEN, "{method} {path}");
                assert_eq!(body["errors"][0]["code"], "forbidden");
            }
            let private = path.replace("contest-one", "other");
            for user in ["reader", "writer"] {
                assert_eq!(
                    request(&app, Some(user), &method, &private, "invalid-json")
                        .await
                        .0,
                    StatusCode::FORBIDDEN,
                    "{user}: {method} {private}"
                );
            }
        }
    }
    assert!(writes > 20);
    assert_eq!(
        serde_json::to_value(store.get_event("contest-one").await.unwrap()).unwrap(),
        before
    );
}

#[tokio::test]
async fn writer_can_create_update_delete_and_recreate_future_events() {
    let (app, _) = fixture().await;
    let body = json!({"name": "contest-future", "problems": [], "teams": [], "score_freeze_time_seconds": 100, "penalty_seconds": 1200, "time_seconds": -60}).to_string();
    let path = "/internal/events/contest-future";
    for _ in 0..2 {
        assert_eq!(
            request(&app, Some("writer"), "POST", path, &body).await.0,
            StatusCode::CREATED
        );
        assert_eq!(
            request(&app, Some("reader"), "GET", path, "").await.0,
            StatusCode::OK
        );
        assert_eq!(
            request(
                &app,
                Some("writer"),
                "PATCH",
                &format!("{path}/time"),
                r#"{"time_seconds":10}"#
            )
            .await
            .0,
            StatusCode::OK
        );
        assert_eq!(
            request(&app, Some("writer"), "DELETE", path, "").await.0,
            StatusCode::NO_CONTENT
        );
    }
}
