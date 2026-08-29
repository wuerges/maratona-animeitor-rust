//! Integration tests for the internal API (`doc/event-api.md`).

use actix_web::{App, http::header, test, web};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use server_v2::internal::{self, InternalToken};
use service::event_store::EventStore;

const TOKEN: &str = "token-de-teste";

fn basic_header() -> (&'static str, String) {
    let auth = BASE64.encode(format!("usuario:{TOKEN}"));
    (header::AUTHORIZATION.as_str(), format!("Basic {auth}"))
}

fn event_body() -> serde_json::Value {
    serde_json::json!({
        "name": "ensaio",
        "problems": ["A", "B"],
        "teams": [
            { "login": "teambr001", "escola": "FACOM - UFMS", "nome": "Time de Teste" }
        ],
        "score_freeze_time_seconds": 2040,
        "penalty_seconds": 1200,
        "time_seconds": -60
    })
}

async fn app() -> impl actix_web::dev::Service<
    actix_http::Request,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    test::init_service(
        App::new()
            .app_data(web::Data::new(EventStore::new()))
            .app_data(web::Data::new(InternalToken(Some(TOKEN.to_string()))))
            .service(web::scope("internal").configure(internal::configure)),
    )
    .await
}

#[actix_web::test]
async fn rejects_missing_credentials() {
    let app = app().await;
    let req = test::TestRequest::get().uri("/internal/events/ensaio").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["errors"][0]["code"], "unauthorized");
}

#[actix_web::test]
async fn rejects_wrong_token() {
    let app = app().await;
    let auth = BASE64.encode("usuario:senha-errada");
    let req = test::TestRequest::get()
        .uri("/internal/events/ensaio")
        .insert_header((header::AUTHORIZATION, format!("Basic {auth}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn event_lifecycle() {
    let app = app().await;
    let (name, auth) = basic_header();

    // Create.
    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio")
        .insert_header((name, auth.clone()))
        .set_json(&event_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["name"], "ensaio");
    assert_eq!(body["data"]["time_seconds"], -60);

    // Read.
    let req = test::TestRequest::get()
        .uri("/internal/events/ensaio")
        .insert_header((name, auth.clone()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

    // Duplicate create conflicts.
    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio")
        .insert_header((name, auth.clone()))
        .set_json(&event_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CONFLICT);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["errors"][0]["code"], "conflict");

    // Delete.
    let req = test::TestRequest::delete()
        .uri("/internal/events/ensaio")
        .insert_header((name, auth.clone()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::NO_CONTENT);

    // Delete again: not found.
    let req = test::TestRequest::delete()
        .uri("/internal/events/ensaio")
        .insert_header((name, auth.clone()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn bad_json_and_missing_field() {
    let app = app().await;
    let (name, auth) = basic_header();

    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio")
        .insert_header((name, auth.clone()))
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_payload("{ não é json")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["errors"][0]["code"], "invalid_json");

    let missing = serde_json::json!({
        "name": "ensaio",
        "problems": ["A"],
        "score_freeze_time_seconds": 2040
        // teams e penalty_seconds ausentes
    });
    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio")
        .insert_header((name, auth.clone()))
        .set_json(&missing)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["errors"][0]["code"], "missing_field");
}

#[actix_web::test]
async fn patch_time_allows_negative_countdown() {
    let app = app().await;
    let (name, auth) = basic_header();

    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio")
        .insert_header((name, auth.clone()))
        .set_json(&event_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    let req = test::TestRequest::patch()
        .uri("/internal/events/ensaio/time")
        .insert_header((name, auth.clone()))
        .set_json(&serde_json::json!({ "time_seconds": -120 }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["time_seconds"], -120);
}

#[actix_web::test]
async fn runs_are_added_and_corrected() {
    let app = app().await;
    let (name, auth) = basic_header();

    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio")
        .insert_header((name, auth.clone()))
        .set_json(&event_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    let runs = serde_json::json!({
        "runs": [
            { "id": 1, "team_login": "teambr001", "prob": "A", "time_seconds": 56, "answer": "N" },
            { "id": 2, "team_login": "teambr001", "prob": "B", "time_seconds": 139, "answer": "Y" }
        ]
    });
    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio/runs")
        .insert_header((name, auth.clone()))
        .set_json(&runs)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["added"], 2);
    assert_eq!(body["data"]["updated"], 0);

    // Same id: judge correction, last value wins.
    let correction = serde_json::json!({
        "runs": [
            { "id": 1, "team_login": "teambr001", "prob": "A", "time_seconds": 56, "answer": "Y" }
        ]
    });
    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio/runs")
        .insert_header((name, auth.clone()))
        .set_json(&correction)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["added"], 0);
    assert_eq!(body["data"]["updated"], 1);

    // Unknown team/prob → 400 invalid_value.
    let bad = serde_json::json!({
        "runs": [
            { "id": 3, "team_login": "desconhecido", "prob": "A", "time_seconds": 1, "answer": "Y" }
        ]
    });
    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio/runs")
        .insert_header((name, auth.clone()))
        .set_json(&bad)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["errors"][0]["code"], "invalid_value");
}

#[actix_web::test]
async fn empty_contest_name_is_rejected() {
    let app = app().await;
    let (name, auth) = basic_header();

    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio")
        .insert_header((name, auth.clone()))
        .set_json(&event_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    // There is no default contest: empty names are rejected on create.
    let contest = serde_json::json!({
        "name": "",
        "codes": ["teambr"]
    });
    let req = test::TestRequest::post()
        .uri("/internal/contests/ensaio/brasil")
        .insert_header((name, auth.clone()))
        .set_json(&contest)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["errors"][0]["code"], "invalid_value");

    // ... and on replace of an existing contest.
    let named = serde_json::json!({ "name": "brasil", "codes": ["teambr"] });
    let req = test::TestRequest::post()
        .uri("/internal/contests/ensaio/brasil")
        .insert_header((name, auth.clone()))
        .set_json(&named)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    let req = test::TestRequest::put()
        .uri("/internal/contests/ensaio/brasil")
        .insert_header((name, auth.clone()))
        .set_json(&serde_json::json!({ "name": "", "codes": ["teambr"] }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["errors"][0]["code"], "invalid_value");
}

#[actix_web::test]
async fn named_contest_and_site_lifecycle() {
    let app = app().await;
    let (name, auth) = basic_header();

    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio")
        .insert_header((name, auth.clone()))
        .set_json(&event_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    let contest = serde_json::json!({
        "name": "brasil",
        "codes": ["teambr"]
    });
    let req = test::TestRequest::post()
        .uri("/internal/contests/ensaio/brasil")
        .insert_header((name, auth.clone()))
        .set_json(&contest)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    let site = serde_json::json!({
        "name": "fiemg",
        "codes": ["teambr"]
    });
    let req = test::TestRequest::post()
        .uri("/internal/sites/ensaio/brasil/fiemg")
        .insert_header((name, auth.clone()))
        .set_json(&site)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    // Deleting the contest removes its sites.
    let req = test::TestRequest::delete()
        .uri("/internal/contests/ensaio/brasil")
        .insert_header((name, auth.clone()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::NO_CONTENT);

    let req = test::TestRequest::get()
        .uri("/internal/events/ensaio/contests/brasil/sites")
        .insert_header((name, auth.clone()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn salt_endpoints_generate_when_body_is_absent() {
    let app = app().await;
    let (name, auth) = basic_header();

    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio")
        .insert_header((name, auth.clone()))
        .set_json(&event_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    let contest = serde_json::json!({ "name": "brasil", "codes": ["teambr"] });
    let req = test::TestRequest::post()
        .uri("/internal/contests/ensaio/brasil")
        .insert_header((name, auth.clone()))
        .set_json(&contest)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    // Explicit salt is returned as-is.
    let req = test::TestRequest::post()
        .uri("/internal/contests/ensaio/brasil/salt")
        .insert_header((name, auth.clone()))
        .set_json(&serde_json::json!({ "salt": "meu-salt" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["salt"], "meu-salt");

    // Missing body generates a random salt.
    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio/salt")
        .insert_header((name, auth.clone()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let salt = body["data"]["salt"].as_str().expect("salt gerado");
    assert_eq!(salt.len(), 32);
    assert!(salt.chars().all(|c| c.is_ascii_alphanumeric()));

    // Salt on a missing site: not found.
    let req = test::TestRequest::post()
        .uri("/internal/sites/ensaio/brasil/inexistente/salt")
        .insert_header((name, auth.clone()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn unknown_answer_is_invalid_value() {
    let app = app().await;
    let (name, auth) = basic_header();

    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio")
        .insert_header((name, auth.clone()))
        .set_json(&event_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    let bad = serde_json::json!({
        "runs": [
            { "id": 1, "team_login": "teambr001", "prob": "A", "time_seconds": 1, "answer": "Z" }
        ]
    });
    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio/runs")
        .insert_header((name, auth.clone()))
        .set_json(&bad)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["errors"][0]["code"], "invalid_value");
}

#[actix_web::test]
async fn salt_endpoints_accept_empty_object_and_empty_salt() {
    let app = app().await;
    let (name, auth) = basic_header();

    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio")
        .insert_header((name, auth.clone()))
        .set_json(&event_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    // Empty object body generates a random salt.
    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio/salt")
        .insert_header((name, auth.clone()))
        .set_json(&serde_json::json!({}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let salt = body["data"]["salt"].as_str().expect("salt gerado");
    assert_eq!(salt.len(), 32);

    // Empty salt value also generates a random salt.
    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio/salt")
        .insert_header((name, auth.clone()))
        .set_json(&serde_json::json!({ "salt": "" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let salt = body["data"]["salt"].as_str().expect("salt gerado");
    assert_eq!(salt.len(), 32);
}

#[actix_web::test]
async fn read_endpoints_list_events_contests_and_sites() {
    let app = app().await;
    let (name, auth) = basic_header();

    // Empty store: events list is empty.
    let req = test::TestRequest::get()
        .uri("/internal/events")
        .insert_header((name, auth.clone()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"], serde_json::json!([]));

    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio")
        .insert_header((name, auth.clone()))
        .set_json(&event_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    let contest = serde_json::json!({
        "name": "brasil",
        "codes": ["teambr"],
        "salt": "salt-do-contest"
    });
    let req = test::TestRequest::post()
        .uri("/internal/contests/ensaio/brasil")
        .insert_header((name, auth.clone()))
        .set_json(&contest)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    let site = serde_json::json!({
        "name": "fiemg",
        "codes": ["teambr"],
        "salt": "salt-do-site"
    });
    let req = test::TestRequest::post()
        .uri("/internal/sites/ensaio/brasil/fiemg")
        .insert_header((name, auth.clone()))
        .set_json(&site)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    // Events.
    let req = test::TestRequest::get()
        .uri("/internal/events")
        .insert_header((name, auth.clone()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"], serde_json::json!(["ensaio"]));

    // Contests, with salts (internal scope).
    let req = test::TestRequest::get()
        .uri("/internal/events/ensaio/contests")
        .insert_header((name, auth.clone()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"][0]["name"], "brasil");
    assert_eq!(body["data"][0]["salt"], "salt-do-contest");

    // Sites, with salts.
    let req = test::TestRequest::get()
        .uri("/internal/events/ensaio/contests/brasil/sites")
        .insert_header((name, auth.clone()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::OK);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"][0]["name"], "fiemg");
    assert_eq!(body["data"][0]["salt"], "salt-do-site");

    // Missing event/contest: not found.
    let req = test::TestRequest::get()
        .uri("/internal/events/inexistente/contests")
        .insert_header((name, auth.clone()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);

    let req = test::TestRequest::get()
        .uri("/internal/events/ensaio/contests/inexistente/sites")
        .insert_header((name, auth.clone()))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::NOT_FOUND);
}

#[actix_web::test]
async fn invalid_regex_is_rejected() {
    let app = app().await;
    let (name, auth) = basic_header();

    let req = test::TestRequest::post()
        .uri("/internal/events/ensaio")
        .insert_header((name, auth.clone()))
        .set_json(&event_body())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::CREATED);

    let bad = serde_json::json!({ "name": "ruim", "codes": ["("] });
    let req = test::TestRequest::post()
        .uri("/internal/contests/ensaio/ruim")
        .insert_header((name, auth.clone()))
        .set_json(&bad)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["errors"][0]["code"], "invalid_regex");
}
