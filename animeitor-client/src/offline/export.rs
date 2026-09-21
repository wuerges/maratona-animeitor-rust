//! Rust export pipeline shared by browser code and native tests.
use super::{css, OfflineMedia, OfflineSnapshot, PAYLOAD_ID};
use base64::{engine::general_purpose::STANDARD, Engine};
use futures::{
    future::{LocalBoxFuture, Shared},
    FutureExt, StreamExt,
};
use serde::Deserialize;
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeSet, HashMap},
    rc::Rc,
};
use url::Url;

#[derive(Clone)]
pub(super) struct Resource {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

pub(super) type Fetch = Rc<dyn Fn(String) -> LocalBoxFuture<'static, Result<Resource, String>>>;
type CachedAsset = Shared<LocalBoxFuture<'static, Option<String>>>;

pub(super) struct Resources {
    fetch: Fetch,
    cache: RefCell<HashMap<String, CachedAsset>>,
    omitted: Rc<RefCell<BTreeSet<String>>>,
    completed: Rc<Cell<usize>>,
    progress: Rc<dyn Fn(String)>,
}

impl Resources {
    pub fn new(fetch: Fetch, progress: Rc<dyn Fn(String)>) -> Self {
        Self {
            fetch,
            cache: RefCell::new(HashMap::new()),
            omitted: Default::default(),
            completed: Default::default(),
            progress,
        }
    }

    pub async fn required(&self, url: &Url) -> Result<Resource, String> {
        (self.fetch)(url.to_string())
            .await
            .map_err(|e| format!("{url}: {e}"))
    }

    async fn asset(&self, url: Url) -> Option<String> {
        let key = url.to_string();
        let cached = self.cache.borrow().get(&key).cloned();
        let future = cached.unwrap_or_else(|| {
            let fetch = self.fetch.clone();
            let omitted = self.omitted.clone();
            let completed = self.completed.clone();
            let progress = self.progress.clone();
            let future = async move {
                let result = match fetch(url.to_string()).await {
                    Ok(resource) => {
                        let hash = url.fragment().map(|s| format!("#{s}")).unwrap_or_default();
                        Some(format!(
                            "{}{}",
                            data_url(&resource.content_type, &resource.bytes),
                            hash
                        ))
                    }
                    Err(error) => {
                        omitted.borrow_mut().insert(format!("{url}: {error}"));
                        None
                    }
                };
                completed.set(completed.get() + 1);
                progress(format!(
                    "Preparing offline file… {} assets processed",
                    completed.get()
                ));
                result
            }
            .boxed_local()
            .shared();
            self.cache.borrow_mut().insert(key, future.clone());
            future
        });
        future.await
    }

    pub fn stylesheet<'a>(
        &'a self,
        url: Url,
        mut ancestors: Vec<Url>,
    ) -> LocalBoxFuture<'a, String> {
        async move {
            if ancestors.contains(&url) {
                return String::new();
            }
            ancestors.push(url.clone());
            match self.required(&url).await {
                Ok(resource) => {
                    self.rewrite_css(&String::from_utf8_lossy(&resource.bytes), &url, ancestors)
                        .await
                }
                Err(error) => {
                    self.omitted.borrow_mut().insert(error);
                    String::new()
                }
            }
        }
        .boxed_local()
    }

    pub fn rewrite_css<'a>(
        &'a self,
        css_text: &'a str,
        base: &'a Url,
        ancestors: Vec<Url>,
    ) -> LocalBoxFuture<'a, String> {
        async move {
            let mut out = String::new();
            let mut previous = 0;
            for reference in css::references(css_text) {
                out.push_str(&css_text[previous..reference.span.start]);
                let replacement = if !reference.import && reference.url.is_empty() {
                    data_url("application/octet-stream", &[])
                } else if !reference.import
                    && (reference.url.starts_with('#') || reference.url.starts_with("data:"))
                {
                    reference.url.clone()
                } else {
                    match base.join(&reference.url) {
                        Ok(url) if reference.import => data_url(
                            "text/css",
                            self.stylesheet(url, ancestors.clone()).await.as_bytes(),
                        ),
                        Ok(url) => self
                            .asset(url)
                            .await
                            .unwrap_or_else(|| data_url("application/octet-stream", &[])),
                        Err(error) => {
                            self.omitted
                                .borrow_mut()
                                .insert(format!("{}: {error}", reference.url));
                            data_url(
                                if reference.import {
                                    "text/css"
                                } else {
                                    "application/octet-stream"
                                },
                                &[],
                            )
                        }
                    }
                };
                out.push_str(&format!(
                    "url({})",
                    serde_json::to_string(&replacement).unwrap()
                ));
                previous = reference.span.end;
            }
            out.push_str(&css_text[previous..]);
            out
        }
        .boxed_local()
    }

    pub async fn media(&self, sources: OfflineMedia, base: &Url) -> OfflineMedia {
        let jobs = sources
            .photos
            .into_iter()
            .map(|(login, url)| (false, login, url))
            .chain(
                sources
                    .sounds
                    .into_iter()
                    .map(|(login, url)| (true, login, url)),
            );
        let downloaded = futures::stream::iter(jobs)
            .map(|(sound, login, source)| async move {
                let value = match base.join(&source) {
                    Ok(url) => self.asset(url).await,
                    Err(error) => {
                        self.omitted
                            .borrow_mut()
                            .insert(format!("{source}: {error}"));
                        None
                    }
                };
                (sound, login, value)
            })
            .buffer_unordered(4)
            .collect::<Vec<_>>()
            .await;
        let mut media = OfflineMedia::default();
        for (sound, login, value) in downloaded {
            if let Some(value) = value {
                if sound {
                    &mut media.sounds
                } else {
                    &mut media.photos
                }
                .insert(login, value);
            }
        }
        media
    }

    pub fn omissions(&self) -> Vec<String> {
        self.omitted.borrow().iter().cloned().collect()
    }
}

pub(super) fn data_url(content_type: &str, bytes: &[u8]) -> String {
    format!("data:{content_type};base64,{}", STANDARD.encode(bytes))
}

#[derive(Deserialize)]
pub(super) struct Manifest {
    pub version: u32,
    pub js: String,
    pub wasm: String,
}

pub(super) struct Stylesheet {
    pub css: String,
    pub media: String,
}

pub(super) struct ExportedFile {
    pub html: String,
    pub filename: String,
    pub message: String,
}

pub(super) fn assemble(
    snapshot: &OfflineSnapshot,
    runtime: &str,
    wasm: &[u8],
    styles: &[Stylesheet],
) -> Result<ExportedFile, String> {
    if regex::Regex::new(r"(?m)^\s*import\s")
        .unwrap()
        .is_match(runtime)
    {
        return Err("Runtime contains unpackaged JavaScript imports".into());
    }
    let payload = STANDARD.encode(serde_json::to_vec(snapshot).map_err(|e| e.to_string())?);
    let style_tags: String = styles
        .iter()
        .map(|style| {
            format!(
                "<link rel=\"stylesheet\" href=\"{}\" media=\"{}\">",
                data_url("text/css", style.css.as_bytes()),
                html_escape::encode_double_quoted_attribute(&style.media)
            )
        })
        .collect();
    // Only bootstrap glue remains in JavaScript. Export logic, data encoding,
    // CSS resolution, styles, and the download itself are all implemented in Rust.
    let runtime = regex::Regex::new(r"(?i)</script")
        .unwrap()
        .replace_all(runtime, r"<\/script");
    let boot = format!("try {{ await __wbg_init({{module_or_path: Uint8Array.from(atob('{}'), c => c.charCodeAt(0))}}); }} catch (error) {{ document.body.textContent = 'Could not open offline reveleitor: ' + error.message; }}", STANDARD.encode(wasm));
    let html = format!("<!doctype html><html><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>Offline reveleitor</title>{style_tags}</head><body id=\"body\"><script id=\"{PAYLOAD_ID}\" type=\"application/octet-stream\">{payload}</script><script type=\"module\">{runtime}\n{boot}</script></body></html>");
    let filename = format!(
        "{}-{}-reveleitor.html",
        snapshot.contest.contest_name, snapshot.sede.name
    )
    .chars()
    .map(|c| {
        if c.is_ascii_control() || "<>:\"/\\|?*".contains(c) {
            '_'
        } else {
            c
        }
    })
    .collect();
    let message = if snapshot.omitted_assets.is_empty() {
        "Offline file saved. Double-click the HTML to open it.".into()
    } else {
        format!(
            "Offline file saved. {} unavailable assets (listed in the saved file):\n{}",
            snapshot.omitted_assets.len(),
            snapshot.omitted_assets.join("\n")
        )
    };
    Ok(ExportedFile {
        html,
        filename,
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;

    fn resources(entries: &[(&str, &str, &str)]) -> (Resources, Rc<RefCell<Vec<String>>>) {
        let fixtures: HashMap<_, _> = entries
            .iter()
            .map(|(url, content_type, text)| {
                (
                    url.to_string(),
                    Resource {
                        bytes: text.as_bytes().to_vec(),
                        content_type: content_type.to_string(),
                    },
                )
            })
            .collect();
        let requests = Rc::new(RefCell::new(vec![]));
        let recorded = requests.clone();
        let fetch: Fetch = Rc::new(move |url| {
            recorded.borrow_mut().push(url.clone());
            let result = fixtures
                .get(&url)
                .cloned()
                .ok_or_else(|| "HTTP 404".to_owned());
            async move { result }.boxed_local()
        });
        (Resources::new(fetch, Rc::new(|_| ())), requests)
    }

    #[test]
    fn css_imports_resolve_recursively_preserve_qualifiers_and_break_cycles() {
        block_on(async {
            let (resources, requests) = resources(&[
                (
                    "https://example.org/css/child.css",
                    "text/css",
                    "@import 'main.css'; .x {background:url(../photo.png)}",
                ),
                ("https://example.org/photo.png", "image/png", "photo"),
            ]);
            let base = Url::parse("https://example.org/css/main.css").unwrap();
            let css = resources
                .rewrite_css(
                    "@import 'child.css' layer(theme) supports(display: grid) screen;",
                    &base,
                    vec![base.clone()],
                )
                .await;
            assert!(css.ends_with(" layer(theme) supports(display: grid) screen;"));
            let encoded = css
                .split("base64,")
                .nth(1)
                .unwrap()
                .split('"')
                .next()
                .unwrap();
            let child = String::from_utf8(STANDARD.decode(encoded).unwrap()).unwrap();
            assert!(child.contains(&data_url("image/png", b"photo")));
            assert!(child.contains("@import url(\"data:text/css;base64,\")"));
            assert_eq!(
                *requests.borrow(),
                [
                    "https://example.org/css/child.css",
                    "https://example.org/photo.png"
                ]
            );
        });
    }

    #[test]
    fn escaped_and_missing_assets_are_embedded_once_with_a_report() {
        block_on(async {
            let (resources, requests) =
                resources(&[("https://example.org/a%20b.png", "image/png", "photo")]);
            let base = Url::parse("https://example.org/css/main.css").unwrap();
            let css = resources.rewrite_css(r#"/* url(fake) */ .a {content: "url(fake)"; background:url('../a\ b.png'),url('../a\ b.png'),url(missing),url(""); mask:url(#mask)}"#, &base, vec![]).await;
            assert!(css.contains("/* url(fake) */"));
            assert!(css.contains("content: \"url(fake)\""));
            assert!(css.contains("url(\"#mask\")"));
            assert!(css.contains("data:application/octet-stream;base64,"));
            assert_eq!(css.matches(&data_url("image/png", b"photo")).count(), 2);
            assert_eq!(requests.borrow().len(), 2);
            assert_eq!(
                resources.omissions(),
                ["https://example.org/css/missing: HTTP 404"]
            );
        });
    }

    #[test]
    fn media_deduplicates_across_teams_and_preserves_available_fallbacks() {
        block_on(async {
            let (resources, requests) =
                resources(&[("https://example.org/shared", "image/png", "photo")]);
            let sources = OfflineMedia {
                photos: [
                    ("a1".into(), "/shared".into()),
                    ("a2".into(), "/missing".into()),
                    ("fake".into(), "/shared".into()),
                ]
                .into(),
                sounds: [("applause".into(), "/missing".into())].into(),
            };
            let media = resources
                .media(sources, &Url::parse("https://example.org/").unwrap())
                .await;
            assert_eq!(media.photos["a1"], media.photos["fake"]);
            assert!(!media.photos.contains_key("a2"));
            assert!(media.sounds.is_empty());
            assert_eq!(resources.omissions().len(), 1);
            assert_eq!(requests.borrow().len(), 2);
        });
    }

    #[test]
    fn html_escapes_untrusted_content_and_roundtrips_unicode_payload() {
        let snapshot = super::super::tests::fixture();
        let styles = [Stylesheet {
            css: "/* </style><script>bad()</script> */ .a{}".into(),
            media: "screen\" onload=\"bad()".into(),
        }];
        let exported =
            assemble(&snapshot, "export const x = '</SCRIPT>';", b"wasm", &styles).unwrap();
        let payload = exported
            .html
            .split("type=\"application/octet-stream\">")
            .nth(1)
            .unwrap()
            .split('<')
            .next()
            .unwrap();
        let restored = OfflineSnapshot::decode(payload).unwrap();
        assert_eq!(restored.contest.contest_name, snapshot.contest.contest_name);
        assert!(!exported.html.contains("DO-NOT-EXPORT"));
        assert_eq!(exported.html.matches("</script>").count(), 2);
        assert!(!exported.html.contains("</SCRIPT>"));
        assert!(exported.html.contains("screen&quot; onload=&quot;bad()"));
        assert!(!exported.filename.contains('/'));
        assert!(assemble(&snapshot, "import x from './snippet.js';", &[], &[]).is_err());
    }
}
