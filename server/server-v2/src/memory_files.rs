use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use axum::Router;
use axum::body::Bytes;
use axum::extract::{OriginalUri, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use mime_guess::Mime;
use sha2::{Digest, Sha256};
use tracing::{info, warn};

/// The static client assets, read from disk once at startup and served
/// entirely from memory afterwards. No filesystem access happens on requests.
pub struct MemoryFiles {
    files: HashMap<String, Asset>,
}

struct Asset {
    raw: Bytes,
    gz: Option<Bytes>,
    br: Option<Bytes>,
    mime: Mime,
    /// hex sha256 of the raw content; the served ETag adds an encoding suffix.
    etag: String,
    /// true when the filename carries a content hash (trunk build output),
    /// making it safe to cache indefinitely.
    is_hashed: bool,
}

impl MemoryFiles {
    /// Loads every file under `dir` into memory, pre-compressing gzip and
    /// brotli variants. Returns an empty store (all requests 404) if the
    /// directory does not exist.
    pub fn load(dir: &Path) -> Self {
        let start = Instant::now();
        let mut files = HashMap::new();

        if !dir.is_dir() {
            warn!(
                "client assets not found at {} — serving no static files",
                dir.display()
            );
            return MemoryFiles { files };
        }

        let mut total_raw = 0usize;
        let mut total_compressed = 0usize;

        let mut stack = vec![dir.to_path_buf()];
        while let Some(current) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&current) else {
                continue;
            };
            for entry in entries.flatten() {
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                let path = entry.path();
                if file_type.is_dir() {
                    stack.push(path);
                } else if file_type.is_file() {
                    let Ok(relative) = path.strip_prefix(dir) else {
                        continue;
                    };
                    let Some(name) = relative.to_str() else {
                        continue;
                    };
                    let Some(asset) = Asset::load(&path, name) else {
                        continue;
                    };
                    total_raw += asset.raw.len();
                    total_compressed += asset.gz.as_ref().map_or(0, |b| b.len())
                        + asset.br.as_ref().map_or(0, |b| b.len());
                    files.insert(name.to_owned(), asset);
                }
            }
        }

        info!(
            "loaded {} client asset(s) from {} into memory ({} raw + {} compressed bytes) in {:?}",
            files.len(),
            dir.display(),
            total_raw,
            total_compressed,
            start.elapsed()
        );

        MemoryFiles { files }
    }
}

/// How one volume mount is served: the URL prefix to strip before the
/// lookup, and whether unmatched paths fall back to `index.html` (SPA).
#[derive(Clone)]
struct MemoryMount {
    assets: Arc<MemoryFiles>,
    mount: String,
    spa_fallback: bool,
}

/// Router serving the in-memory assets, meant to be merged into the app.
/// `mount` is the URL prefix the volume is mounted at (`""` for the root
/// mount); `spa_fallback` serves `index.html` for unmatched paths, which is
/// how the client SPA handles `/animeitor/{event}/{contest}` routes.
pub fn router(assets: Arc<MemoryFiles>, mount: &str, spa_fallback: bool) -> Router {
    Router::new().fallback(serve).with_state(MemoryMount {
        assets,
        mount: mount.to_string(),
        spa_fallback,
    })
}

async fn serve(State(mount): State<MemoryMount>, uri: OriginalUri, headers: HeaderMap) -> Response {
    let path = uri.path();
    let name = if mount.mount.is_empty() {
        path.trim_start_matches('/')
    } else if path == mount.mount {
        ""
    } else if let Some(rest) = path.strip_prefix(&format!("{}/", mount.mount)) {
        rest
    } else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let name = if name.is_empty() { "index.html" } else { name };

    let asset = mount
        .assets
        .files
        .get(name)
        .or_else(|| {
            mount
                .spa_fallback
                .then(|| mount.assets.files.get("index.html"))
                .flatten()
        });
    let Some(asset) = asset else {
        return StatusCode::NOT_FOUND.into_response();
    };

    // None means identity (the raw bytes).
    let encoding = negotiate(
        headers
            .get(header::ACCEPT_ENCODING)
            .and_then(|value| value.to_str().ok()),
        asset,
    );

    // Different encodings are different representations, so the ETag gets a suffix.
    let etag = match encoding {
        Some(Encoding::Br) => format!("{}-br", asset.etag),
        Some(Encoding::Gzip) => format!("{}-gz", asset.etag),
        None => asset.etag.clone(),
    };

    let mut response = Response::new(axum::body::Body::empty());
    {
        let response_headers = response.headers_mut();
        response_headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static(if asset.is_hashed {
                "public, max-age=31536000, immutable"
            } else {
                "no-cache"
            }),
        );
        response_headers.insert(header::VARY, HeaderValue::from_static("Accept-Encoding"));
        response_headers.insert(
            header::ETAG,
            HeaderValue::from_str(&quoted(&etag)).expect("quoted hex etag is a valid header value"),
        );
    }

    let if_none_match = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok());
    if let Some(if_none_match) = if_none_match
        && etag_matches(if_none_match, &etag)
    {
        *response.status_mut() = StatusCode::NOT_MODIFIED;
        return response;
    }

    *response.status_mut() = StatusCode::OK;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(asset.mime.as_ref()).expect("mime_guess produces valid header values"),
    );
    let body = match encoding {
        Some(Encoding::Br) => {
            response
                .headers_mut()
                .insert(header::CONTENT_ENCODING, HeaderValue::from_static("br"));
            asset.br.clone().expect("br variant exists")
        }
        Some(Encoding::Gzip) => {
            response.headers_mut().insert(
                header::CONTENT_ENCODING,
                HeaderValue::from_static("gzip"),
            );
            asset.gz.clone().expect("gzip variant exists")
        }
        None => asset.raw.clone(),
    };
    *response.body_mut() = axum::body::Body::from(body);

    response
}

/// Picks the best available variant for the client's Accept-Encoding.
/// `None` means the raw bytes. Falls back to identity when nothing
/// acceptable matches (RFC 9110 allows serving identity in that case).
fn negotiate(accept_encoding: Option<&str>, asset: &Asset) -> Option<Encoding> {
    let accept_encoding = accept_encoding?;

    let mut br_q = 0.0f32;
    let mut gzip_q = 0.0f32;
    for entry in accept_encoding.split(',') {
        let mut parts = entry.trim().split(';');
        let coding = parts.next().unwrap_or_default().trim().to_ascii_lowercase();
        let mut q = 1.0f32;
        for param in parts {
            if let Some(value) = param.trim().strip_prefix("q=") {
                q = value.parse().unwrap_or(1.0);
            }
        }
        match coding.as_str() {
            "br" => br_q = br_q.max(q),
            "gzip" | "x-gzip" => gzip_q = gzip_q.max(q),
            "*" => {
                br_q = br_q.max(q);
                gzip_q = gzip_q.max(q);
            }
            _ => {}
        }
    }

    if asset.br.is_some() && br_q > 0.0 {
        Some(Encoding::Br)
    } else if asset.gz.is_some() && gzip_q > 0.0 {
        Some(Encoding::Gzip)
    } else {
        None
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Encoding {
    Br,
    Gzip,
}

impl Asset {
    fn load(path: &Path, name: &str) -> Option<Asset> {
        let raw = std::fs::read(path).ok()?;
        let etag = format!("{:x}", Sha256::digest(&raw));
        let gz = gzip(&raw);
        let br = brotli(&raw);
        Some(Asset {
            raw: Bytes::from(raw),
            gz: gz.map(Bytes::from),
            br: br.map(Bytes::from),
            mime: mime_guess::from_path(name).first_or_octet_stream(),
            etag,
            is_hashed: is_hashed_filename(name),
        })
    }
}

fn etag_matches(if_none_match: &str, etag: &str) -> bool {
    if_none_match.split(',').any(|candidate| {
        let candidate = candidate.trim().trim_start_matches("W/").trim_matches('"');
        candidate == "*" || candidate == etag
    })
}

fn quoted(etag: &str) -> String {
    format!("\"{etag}\"")
}

/// Trunk (and bundlers generally) give compiled assets a content hash in the
/// filename, e.g. `client-v2-82d7749ea33c3f8d_bg.wasm` or
/// `styles-48e01c3f2adb8d51.css`. Those names never change content, so they
/// can be cached forever.
fn is_hashed_filename(name: &str) -> bool {
    let file_name = name.rsplit('/').next().unwrap_or(name);
    let stem = file_name.split('.').next().unwrap_or(file_name);
    let Some(dash) = stem.rfind('-') else {
        return false;
    };
    let rest = &stem[dash + 1..];
    rest.len() >= 16
        && rest[..16].bytes().all(|b| b.is_ascii_hexdigit())
        && rest[16..].chars().next().is_none_or(|c| c == '.' || c == '_' || c == '-')
}

fn gzip(bytes: &[u8]) -> Option<Vec<u8>> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
    encoder.write_all(bytes).ok()?;
    encoder.finish().ok()
}

fn brotli(bytes: &[u8]) -> Option<Vec<u8>> {
    let params = brotli::enc::BrotliEncoderParams {
        quality: 7,
        ..Default::default()
    };
    let mut input = bytes;
    let mut output = Vec::new();
    brotli::BrotliCompress(&mut input, &mut output, &params).ok()?;
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(gz: bool, br: bool) -> Asset {
        Asset {
            raw: Bytes::new(),
            gz: gz.then(Bytes::new),
            br: br.then(Bytes::new),
            mime: mime_guess::from_path("x.txt").first_or_octet_stream(),
            etag: "abc123".to_string(),
            is_hashed: false,
        }
    }

    #[test]
    fn test_negotiate_prefers_brotli() {
        let a = asset(true, true);
        assert!(matches!(negotiate(Some("gzip, br"), &a), Some(Encoding::Br)));
        // Like the actix-era server, brotli wins whenever it has any positive
        // q, even when gzip was preferred: both are acceptable to the client.
        assert!(matches!(negotiate(Some("br;q=0.5, gzip;q=1"), &a), Some(Encoding::Br)));
        // Without a brotli variant, gzip serves.
        let a = asset(true, false);
        assert!(matches!(negotiate(Some("br, gzip"), &a), Some(Encoding::Gzip)));
    }

    #[test]
    fn test_negotiate_falls_back_to_identity() {
        let a = asset(true, true);
        assert!(negotiate(None, &a).is_none());
        assert!(negotiate(Some("identity"), &a).is_none());
        let a = asset(false, false);
        assert!(negotiate(Some("gzip, br"), &a).is_none());
    }

    #[test]
    fn test_etag_matches() {
        assert!(etag_matches("\"abc123\"", "abc123"));
        assert!(etag_matches("W/\"abc123\"", "abc123"));
        assert!(etag_matches("\"xyz\", \"abc123\"", "abc123"));
        assert!(etag_matches("*", "abc123"));
        assert!(!etag_matches("\"xyz\"", "abc123"));
    }

    #[test]
    fn test_is_hashed_filename() {
        assert!(is_hashed_filename("styles-48e01c3f2adb8d51.css"));
        assert!(is_hashed_filename("dir/client-v2-82d7749ea33c3f8d_bg.wasm"));
        assert!(is_hashed_filename("audio-9273674b3492b75f.css"));
        assert!(!is_hashed_filename("index.html"));
        assert!(!is_hashed_filename("user-styles.css"));
        assert!(!is_hashed_filename("styles-short.css"));
    }
}
