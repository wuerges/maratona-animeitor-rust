//! Integration tests for the in-memory client asset serving.

use std::io::Read;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use http_body_util::BodyExt;
use server_v2::memory_files::{self, MemoryFiles};
use tower::ServiceExt;

static DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    // The counter guards against two parallel tests landing on the same
    // nanosecond, which would make them share (and delete) each other's dir.
    let counter = DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{name}-{}-{nanos}-{counter}", std::process::id()))
}

fn fixture() -> PathBuf {
    let dir = temp_dir("memory-files-test");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.html"), "<html>oi</html>").unwrap();
    std::fs::write(
        dir.join("styles-48e01c3f2adb8d51.css"),
        "body { color: red }",
    )
    .unwrap();
    dir
}

fn assets(dir: &PathBuf) -> Arc<MemoryFiles> {
    Arc::new(MemoryFiles::load(dir))
}

async fn get(app: Router, uri: &str, extra_headers: &[(&str, &str)]) -> axum::http::Response<Body> {
    let mut request = Request::builder().method(Method::GET).uri(uri);
    for (name, value) in extra_headers {
        request = request.header(*name, *value);
    }
    app.oneshot(request.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn body_bytes(response: axum::http::Response<Body>) -> axum::body::Bytes {
    response.into_body().collect().await.unwrap().to_bytes()
}

#[tokio::test]
async fn test_serves_index_from_memory() {
    let dir = fixture();
    let app = memory_files::router(assets(&dir), "", false);

    let response = get(app, "/", &[]).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key(header::ETAG));
    assert_eq!(
        response.headers().get(header::CACHE_CONTROL).unwrap(),
        "no-cache"
    );
    assert_eq!(&body_bytes(response).await[..], b"<html>oi</html>");

    std::fs::remove_dir_all(&dir).unwrap();
}

#[tokio::test]
async fn test_missing_file_is_404() {
    let dir = fixture();
    let app = memory_files::router(assets(&dir), "", false);

    let response = get(app, "/nao-existe", &[]).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    std::fs::remove_dir_all(&dir).unwrap();
}

#[tokio::test]
async fn test_serves_gzip_variant_and_revalidates() {
    let dir = fixture();
    let app = memory_files::router(assets(&dir), "", false);

    let response = get(
        app.clone(),
        "/styles-48e01c3f2adb8d51.css",
        &[("accept-encoding", "gzip")],
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_ENCODING).unwrap(),
        "gzip"
    );
    assert_eq!(
        response.headers().get(header::CACHE_CONTROL).unwrap(),
        "public, max-age=31536000, immutable"
    );
    let etag = response.headers().get(header::ETAG).unwrap().clone();
    let encoded = body_bytes(response).await;
    let mut decoder = flate2::read::GzDecoder::new(&encoded[..]);
    let mut decoded = String::new();
    decoder.read_to_string(&mut decoded).unwrap();
    assert_eq!(decoded, "body { color: red }");

    // Same representation, matching If-None-Match: 304, no body.
    let response = get(
        app,
        "/styles-48e01c3f2adb8d51.css",
        &[
            ("accept-encoding", "gzip"),
            ("if-none-match", etag.to_str().unwrap()),
        ],
    )
    .await;
    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    assert!(body_bytes(response).await.is_empty());

    std::fs::remove_dir_all(&dir).unwrap();
}

#[tokio::test]
async fn test_spa_mount_falls_back_to_index() {
    let dir = fixture();
    let app = Router::new().nest(
        "/animeitor",
        memory_files::router(assets(&dir), "/animeitor", true),
    );

    // Deep client route: served the SPA index.html.
    let response = get(app.clone(), "/animeitor/evento/contest/", &[]).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(&body_bytes(response).await[..], b"<html>oi</html>");

    // Known asset under the mount is served as itself.
    let response = get(app.clone(), "/animeitor/styles-48e01c3f2adb8d51.css", &[]).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(&body_bytes(response).await[..], b"body { color: red }");

    // The mount itself is the index.
    let response = get(app, "/animeitor", &[]).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(&body_bytes(response).await[..], b"<html>oi</html>");

    std::fs::remove_dir_all(&dir).unwrap();
}
