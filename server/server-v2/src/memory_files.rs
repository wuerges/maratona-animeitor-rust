use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::time::Instant;

use actix_web::{
    http::header,
    web::{self, Bytes},
    HttpRequest, HttpResponse,
};
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

/// Registers the in-memory client assets at the server root. Must be
/// registered after the `api` scope and the disk volumes so they take
/// precedence. The `MemoryFiles` store itself is made available to the
/// handler as app data.
pub fn client_service() -> actix_web::Resource {
    web::resource("/{tail:.*}").to(serve)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Encoding {
    Br,
    Gzip,
}

impl Encoding {
    fn header_value(self) -> &'static str {
        match self {
            Encoding::Br => "br",
            Encoding::Gzip => "gzip",
        }
    }
}

async fn serve(req: HttpRequest, assets: web::Data<MemoryFiles>) -> HttpResponse {
    let name = req.path().trim_start_matches('/');
    let name = if name.is_empty() { "index.html" } else { name };

    let Some(asset) = assets.files.get(name) else {
        return HttpResponse::NotFound().finish();
    };

    // None means identity (the raw bytes).
    let encoding = negotiate(
        req.headers()
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
    let cache_control = if asset.is_hashed {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };

    if let Some(if_none_match) = req
        .headers()
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
    {
        if etag_matches(if_none_match, &etag) {
            return HttpResponse::NotModified()
                .insert_header((header::ETAG, quoted(&etag)))
                .insert_header((header::CACHE_CONTROL, cache_control))
                .insert_header((header::VARY, "Accept-Encoding"))
                .finish();
        }
    }

    let mut response = HttpResponse::Ok();
    response
        .insert_header((header::CONTENT_TYPE, asset.mime.clone()))
        .insert_header((header::CACHE_CONTROL, cache_control))
        .insert_header((header::ETAG, quoted(&etag)))
        .insert_header((header::VARY, "Accept-Encoding"));
    let body = match encoding {
        Some(Encoding::Br) => {
            response.insert_header((header::CONTENT_ENCODING, Encoding::Br.header_value()));
            asset.br.clone().expect("br variant exists")
        }
        Some(Encoding::Gzip) => {
            response.insert_header((header::CONTENT_ENCODING, Encoding::Gzip.header_value()));
            asset.gz.clone().expect("gzip variant exists")
        }
        None => asset.raw.clone(),
    };
    response.body(body)
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
    let mut params = brotli::enc::BrotliEncoderParams::default();
    params.quality = 7;
    let mut input = &bytes[..];
    let mut output = Vec::new();
    brotli::BrotliCompress(&mut input, &mut output, &params).ok()?;
    Some(output)
}
