//! Executable checks of the served contracts and their setup examples.
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use server_v2::{
    AppState, app,
    openapi::{InternalApiDoc, PublicApiDoc},
};
use tower::ServiceExt;
use utoipa::OpenApi;

fn application() -> Router {
    app(AppState {
        public_url: "https://public.example:8443/ignored/base/".parse().unwrap(),
        store: test_store(Some("private-test-secret".into())),
        internal_tokens: std::sync::Arc::new(std::collections::HashMap::from([(
            "operator".into(),
            "test-token".into(),
        )])),
    })
}
fn basic() -> String {
    format!("Basic {}", STANDARD.encode("operator:test-token"))
}
async fn request(
    app: &Router,
    method: &str,
    path: &str,
    body: Option<Value>,
    auth: Option<String>,
) -> (StatusCode, axum::http::HeaderMap, Value) {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(auth) = auth {
        builder = builder.header("Authorization", auth);
    }
    let body = match body {
        Some(value) => {
            builder = builder.header("Content-Type", "application/json");
            Body::from(value.to_string())
        }
        None => Body::empty(),
    };
    let response = app
        .clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, headers, body)
}
fn example(spec: &Value, path: &str, method: &str) -> Value {
    spec["paths"][path][method]["requestBody"]["content"]["application/json"]["example"].clone()
}
fn check_refs(value: &Value, root: &Value) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if key == "$ref" {
                    let pointer = value.as_str().unwrap().strip_prefix('#').unwrap();
                    assert!(root.pointer(pointer).is_some(), "unresolved {pointer}");
                } else {
                    check_refs(value, root);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                check_refs(value, root);
            }
        }
        _ => {}
    }
}

#[test]
fn specs_have_complete_distinct_operations_and_resolved_schemas() {
    let internal = serde_json::to_value(InternalApiDoc::openapi()).unwrap();
    let public = serde_json::to_value(PublicApiDoc::openapi()).unwrap();
    let mut ids = std::collections::HashSet::new();
    for spec in [&internal, &public] {
        check_refs(spec, spec);
        assert!(spec["info"]["description"].as_str().unwrap().len() > 1000);
        for (path, item) in spec["paths"].as_object().unwrap() {
            for (method, operation) in item.as_object().unwrap() {
                assert!(ids.insert(operation["operationId"].as_str().unwrap().to_owned()));
                if !path.ends_with("/metrics") {
                    assert!(
                        operation["responses"]["503"]["description"]
                            .as_str()
                            .unwrap()
                            .contains("storage_unavailable")
                    );
                    assert!(
                        operation["responses"]["500"]["description"]
                            .as_str()
                            .unwrap()
                            .contains("storage_error")
                    );
                }
                assert!(!operation["summary"].as_str().unwrap().is_empty());
                assert!(operation["description"].as_str().unwrap().len() > 40);
                if matches!(method.as_str(), "post" | "put" | "patch") {
                    assert!(
                        operation["requestBody"]["content"]["application/json"]["schema"]
                            .is_object(),
                        "{method} {path}"
                    );
                    assert!(
                        operation["requestBody"]["content"]["application/json"]["example"]
                            .is_object()
                    );
                }
                for (status, response) in operation["responses"].as_object().unwrap() {
                    if ["101", "204"].contains(&status.as_str()) {
                        assert!(response.get("content").is_none());
                    } else if !path.ends_with("metrics")
                        && !path.ends_with("runs_ws")
                        && !path.ends_with("timer")
                        && !path.contains("remote_control")
                        && status != "426"
                    {
                        assert!(
                            response["content"]["application/json"]["schema"].is_object(),
                            "{method} {path} {status}"
                        );
                    }
                }
            }
        }
    }
    assert_eq!(ids.len(), 42);
    let methods: Vec<_> = internal["paths"]["/internal/contests/{event_name}/{contest_name}"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(methods, ["delete", "get", "patch", "post", "put"]);
    assert_eq!(internal["security"], json!([{"basicAuth": []}]));
    assert_eq!(
        internal["components"]["securitySchemes"]["basicAuth"]["scheme"],
        "basic"
    );
    assert!(public.get("security").is_none());
    assert_eq!(
        public["paths"]["/api/events/{event_name}/contests/{contest_name}/runs_secret"]["get"]["security"],
        json!([{"bearerAuth": []}])
    );
    assert_ne!(
        internal["paths"]["/internal/events/{event_name}/salt"]["post"]["requestBody"]["required"],
        true
    );
    // Pin request requiredness/defaults to serde behavior: time and salts can be omitted.
    let schema = &internal["components"]["schemas"]["EventState"];
    assert!(
        !schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("time_seconds"))
    );
    assert!(
        !schema["required"]
            .as_array()
            .unwrap()
            .contains(&json!("salt"))
    );
    let _: data::event::EventState =
        serde_json::from_value(example(&internal, "/internal/events/{event_name}", "post"))
            .unwrap();
    let _: data::event::ContestConfig = serde_json::from_value(example(
        &internal,
        "/internal/contests/{event_name}/{contest_name}",
        "post",
    ))
    .unwrap();
    let _: data::event::SiteConfig = serde_json::from_value(example(
        &internal,
        "/internal/sites/{event_name}/{contest_name}/{site_name}",
        "post",
    ))
    .unwrap();
}

#[tokio::test]
async fn documented_setup_produces_working_revelation_links() {
    let app = application();
    let (status, _, spec) =
        request(&app, "GET", "/internal/openapi.json", None, Some(basic())).await;
    assert_eq!(status, 200);
    let event_path = "/internal/events/regional-2026";
    let (_, _, empty) = request(&app, "GET", "/internal/events", None, Some(basic())).await;
    assert_eq!(empty, json!({"data":[]}));
    for (template, path) in [
        ("/internal/events/{event_name}", event_path),
        (
            "/internal/contests/{event_name}/{contest_name}",
            "/internal/contests/regional-2026/brasil",
        ),
        (
            "/internal/sites/{event_name}/{contest_name}/{site_name}",
            "/internal/sites/regional-2026/brasil/fiemg",
        ),
    ] {
        let (status, _, _) = request(
            &app,
            "POST",
            path,
            Some(example(&spec, template, "post")),
            Some(basic()),
        )
        .await;
        assert_eq!(status, 201, "{path}");
    }
    let listing = "/internal/events/regional-2026/revelation_urls";
    let (status, headers, links) = request(&app, "GET", listing, None, Some(basic())).await;
    assert_eq!(status, 200);
    assert_eq!(headers["cache-control"], "no-store");
    let link: url::Url = links["data"][0]["url"].as_str().unwrap().parse().unwrap();
    assert_eq!(
        link.origin().ascii_serialization(),
        "https://public.example:8443"
    );
    assert_eq!(link.path(), "/animeitor/regional-2026/brasil/");
    assert_eq!(links["data"][0]["contest"], "brasil");
    assert_eq!(links["data"][0]["site"], "fiemg");
    let params: std::collections::HashMap<_, _> = link.query_pairs().into_owned().collect();
    assert_eq!(params["sede"], "fiemg");
    let token = format!("Bearer {}", params["secret"]);
    let secret_path = "/api/events/regional-2026/contests/brasil/runs_secret";
    let (status, _, body) = request(&app, "GET", secret_path, None, Some(token.clone())).await;
    assert_eq!(status, 403);
    assert_eq!(body["errors"][0]["code"], "not_started");
    let runs_path = format!("{event_path}/runs");
    let runs = example(&spec, "/internal/events/{event_name}/runs", "post");
    let (status, _, body) =
        request(&app, "POST", &runs_path, Some(runs.clone()), Some(basic())).await;
    assert_eq!(status, 200);
    assert_eq!(body, json!({"data":{"added":1,"updated":0}}));
    let (_, _, body) = request(&app, "POST", &runs_path, Some(runs.clone()), Some(basic())).await;
    assert_eq!(body, json!({"data":{"added":0,"updated":0}}));
    let mut corrected = runs.clone();
    corrected["runs"][0]["answer"] = json!("N");
    let (_, _, body) = request(&app, "POST", &runs_path, Some(corrected), Some(basic())).await;
    assert_eq!(body, json!({"data":{"added":0,"updated":1}}));
    let mut unknown = runs.clone();
    unknown["runs"][0]["team_login"] = json!("unknown-team");
    let (_, _, body) = request(&app, "POST", &runs_path, Some(unknown), Some(basic())).await;
    assert_eq!(body["warnings"][0]["code"], "unknown_team");
    let (status, _, _) = request(
        &app,
        "PATCH",
        &format!("{event_path}/time"),
        Some(example(
            &spec,
            "/internal/events/{event_name}/time",
            "patch",
        )),
        Some(basic()),
    )
    .await;
    assert_eq!(status, 200);
    for suffix in ["contest", "config"] {
        let (status, _, body) = request(
            &app,
            "GET",
            &format!("/api/events/regional-2026/contests/brasil/{suffix}"),
            None,
            None,
        )
        .await;
        assert_eq!(status, 200);
        assert!(!body.to_string().contains(&params["secret"]));
        assert!(!body.to_string().contains("salt"));
    }
    let (status, _, body) = request(&app, "GET", secret_path, None, Some(token)).await;
    assert_eq!(status, 200);
    assert_eq!(body["data"]["runs"][0]["answer"], "N");
    for salt_path in [
        "/internal/sites/regional-2026/brasil/fiemg/salt",
        "/internal/contests/regional-2026/brasil/salt",
        "/internal/events/regional-2026/salt",
    ] {
        let (_, _, before) = request(&app, "GET", listing, None, Some(basic())).await;
        let before: url::Url = before["data"][0]["url"].as_str().unwrap().parse().unwrap();
        let old = before
            .query_pairs()
            .find(|(key, _)| key == "secret")
            .unwrap()
            .1
            .into_owned();
        let (status, _, _) = request(
            &app,
            "POST",
            salt_path,
            Some(json!({"salt":"rotated"})),
            Some(basic()),
        )
        .await;
        assert_eq!(status, 200);
        let (status, _, body) = request(
            &app,
            "GET",
            secret_path,
            None,
            Some(format!("Bearer {old}")),
        )
        .await;
        assert_eq!(status, 403);
        assert_eq!(body["errors"][0]["code"], "invalid_key");
        let (_, _, after) = request(&app, "GET", listing, None, Some(basic())).await;
        let after: url::Url = after["data"][0]["url"].as_str().unwrap().parse().unwrap();
        let new = after
            .query_pairs()
            .find(|(key, _)| key == "secret")
            .unwrap()
            .1
            .into_owned();
        assert_ne!(new, old);
        let (status, _, _) = request(
            &app,
            "GET",
            secret_path,
            None,
            Some(format!("Bearer {new}")),
        )
        .await;
        assert_eq!(status, 200);
    }
}

#[tokio::test]
async fn listing_handles_credentials_missing_and_empty_events() {
    let app = application();
    let path = "/internal/events/missing/revelation_urls";
    for auth in [
        None,
        Some(format!(
            "Basic {}",
            STANDARD.encode("wrong-user:test-token")
        )),
        Some(format!("Basic {}", STANDARD.encode("operator:wrong-token"))),
    ] {
        let (status, headers, body) = request(&app, "GET", path, None, auth).await;
        assert_eq!(status, 401);
        assert_eq!(headers["www-authenticate"], "Basic");
        assert_eq!(headers["cache-control"], "no-store");
        assert_eq!(body["errors"][0]["code"], "unauthorized");
    }
    let (status, headers, body) = request(&app, "GET", path, None, Some(basic())).await;
    assert_eq!(status, 404);
    assert_eq!(headers["cache-control"], "no-store");
    assert_eq!(body["errors"][0]["code"], "not_found");
    let spec = serde_json::to_value(InternalApiDoc::openapi()).unwrap();
    request(
        &app,
        "POST",
        "/internal/events/regional-2026",
        Some(example(&spec, "/internal/events/{event_name}", "post")),
        Some(basic()),
    )
    .await;
    let (status, _, body) = request(
        &app,
        "GET",
        "/internal/events/regional-2026/revelation_urls",
        None,
        Some(basic()),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(body, json!({"data":[]}));
}

#[tokio::test]
async fn documentation_routes_serve_html_and_protect_internal_spec() {
    let app = application();
    for path in ["/internal/docs", "/internal/openapi.json"] {
        let (status, _, _) = request(&app, "GET", path, None, None).await;
        assert_eq!(status, 401);
    }
    let (status, _, public) = request(&app, "GET", "/api/openapi.json", None, None).await;
    assert_eq!(status, 200);
    assert_eq!(public["info"]["title"], "Animeitor public API");
    for (path, spec, auth) in [
        ("/internal/docs", "/internal/openapi.json", Some(basic())),
        ("/api/docs", "/api/openapi.json", None),
    ] {
        let mut builder = Request::builder().uri(path);
        if let Some(auth) = auth {
            builder = builder.header("Authorization", auth);
        }
        let response = app
            .clone()
            .oneshot(builder.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), 200);
        assert!(
            response.headers()["content-type"]
                .to_str()
                .unwrap()
                .starts_with("text/html")
        );
        let html = String::from_utf8(
            response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        )
        .unwrap();
        assert!(html.contains("SwaggerUIBundle"));
        assert!(html.contains(&format!("url:\"{spec}\"")));
    }
}

#[test]
fn success_examples_include_actual_serialized_defaults() {
    fn roundtrip<T: serde::de::DeserializeOwned + serde::Serialize>(example: &Value) {
        let parsed: T = serde_json::from_value(example.clone()).unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap(), *example);
    }
    let internal = serde_json::to_value(InternalApiDoc::openapi()).unwrap();
    for (path, item) in internal["paths"].as_object().unwrap() {
        for operation in item.as_object().unwrap().values() {
            for status in ["200", "201"] {
                let data = &operation["responses"][status]["content"]["application/json"]["example"]
                    ["data"];
                if data.is_null() {
                    continue;
                }
                match path.as_str() {
                    "/internal/events/{event_name}" => roundtrip::<data::event::EventState>(data),
                    "/internal/contests/{event_name}/{contest_name}" => {
                        roundtrip::<data::event::ContestConfig>(data)
                    }
                    "/internal/sites/{event_name}/{contest_name}/{site_name}" => {
                        roundtrip::<data::event::SiteConfig>(data)
                    }
                    "/internal/events/{event_name}/contests" => {
                        roundtrip::<Vec<data::event::ContestConfig>>(data)
                    }
                    "/internal/events/{event_name}/contests/{contest_name}/sites" => {
                        roundtrip::<Vec<data::event::SiteConfig>>(data)
                    }
                    "/internal/events/{event_name}/revelation_urls" => {
                        roundtrip::<Vec<data::event::RevelationUrl>>(data)
                    }
                    _ => {}
                }
            }
        }
    }
    let schemas = &internal["components"]["schemas"];
    assert_eq!(
        schemas["EventState"]["properties"]["time_seconds"]["default"],
        0
    );
    for (field, value) in [("ouro", 1), ("prata", 2), ("bronze", 3)] {
        assert_eq!(
            schemas["ContestConfig"]["properties"][field]["default"],
            value
        );
    }
    let public = serde_json::to_value(PublicApiDoc::openapi()).unwrap();
    let response = |path: &str| {
        public["paths"][path]["get"]["responses"]["200"]["content"]["application/json"]["example"]["data"].clone()
    };
    roundtrip::<data::event::PublicContestState>(&response(
        "/api/events/{event_name}/contests/{contest_name}/contest",
    ));
    roundtrip::<data::event::PublicConfig>(&response(
        "/api/events/{event_name}/contests/{contest_name}/config",
    ));
    roundtrip::<data::event::RunsData>(&response(
        "/api/events/{event_name}/contests/{contest_name}/runs_secret",
    ));
}

mod common;
use common::test_store;
