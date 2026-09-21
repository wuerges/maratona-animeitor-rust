//! Trunk post-build hook for the standalone reveleitor exporter.
use std::{collections::HashMap, env, error::Error, fs, path::PathBuf};

use regex::Regex;
use serde_json::json;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq)]
struct Assets {
    js: String,
    wasm: String,
    manifest_url: String,
}

impl Assets {
    fn from_html(html: &str) -> Result<Self> {
        // Parse only Trunk's generated link tags, not arbitrary user HTML.
        // Account for attribute ordering, quoting, and escaped public URLs.
        let links = Regex::new(r#"(?is)<link\b(?:"[^"]*"|'[^']*'|[^'">])*>"#)?;
        let attributes = Regex::new(r#"([\w:-]+)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+))"#)?;
        let mut js = vec![];
        let mut wasm = vec![];
        for link in links.find_iter(html) {
            let attrs: HashMap<_, _> = attributes
                .captures_iter(link.as_str())
                .map(|captures| {
                    let value = (2..=4).find_map(|i| captures.get(i)).unwrap();
                    (
                        captures[1].to_ascii_lowercase(),
                        html_escape::decode_html_entities(value.as_str()).into_owned(),
                    )
                })
                .collect();
            if let (Some(rel), Some(href)) = (attrs.get("rel"), attrs.get("href")) {
                match rel.as_str() {
                    "modulepreload" if href.ends_with(".js") => js.push(href.clone()),
                    "preload" if href.ends_with(".wasm") => wasm.push(href.clone()),
                    _ => (),
                }
            }
        }
        let ([js], [wasm]) = (js.as_slice(), wasm.as_slice()) else {
            return Err("Offline export requires exactly one client JS/WASM pair".into());
        };
        let manifest_url = match js.rsplit_once('/') {
            Some((prefix, _)) => format!("{prefix}/offline-manifest.json"),
            None => "offline-manifest.json".into(),
        };
        Ok(Self {
            js: js.rsplit('/').next().unwrap().into(),
            wasm: wasm.rsplit('/').next().unwrap().into(),
            manifest_url,
        })
    }

    fn manifest(&self) -> String {
        json!({"version": 1, "js": self.js, "wasm": self.wasm}).to_string()
    }

    fn install_manifest(&self, html: &str, runtime: &str) -> Result<String> {
        // Inline modules cannot resolve external snippets from file://.
        if Regex::new(r"(?m)^\s*import\s")?.is_match(runtime) {
            return Err("Offline export requires bundling the new JS imports first".into());
        }
        let closing_body = Regex::new(r"(?i)</body\s*>")?;
        let body_end = closing_body
            .find(html)
            .ok_or("Trunk index.html has no closing body tag")?;
        let manifest_url = html_escape::encode_double_quoted_attribute(&self.manifest_url);
        let metadata =
            format!("<meta name=\"reveleitor-offline-manifest\" content=\"{manifest_url}\">\n");
        let mut output = html.to_owned();
        output.insert_str(body_end.start(), &metadata);
        Ok(output)
    }
}

fn main() -> Result<()> {
    let staging = PathBuf::from(
        env::var_os("TRUNK_STAGING_DIR")
            .ok_or("TRUNK_STAGING_DIR is required; run through Trunk")?,
    );
    let index = staging.join("index.html");
    let html = fs::read_to_string(&index)?;
    let assets = Assets::from_html(&html)?;
    let runtime = fs::read_to_string(staging.join(&assets.js))?;
    if !staging.join(&assets.wasm).is_file() {
        return Err("Trunk's referenced WASM asset is missing".into());
    }
    let output = assets.install_manifest(&html, &runtime)?;
    fs::write(staging.join("offline-manifest.json"), assets.manifest())?;
    fs::write(index, output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index(prefix: &str) -> String {
        format!(
            r#"<html><head><link href="{prefix}client.js" rel="modulepreload"><link rel='preload' href='{prefix}client_bg.wasm'></head><body></body></html>"#
        )
    }

    #[test]
    fn finds_runtime_for_relative_root_nested_and_absolute_public_urls() {
        for prefix in ["", "/", "/animeitor/", "https://example.test/app/"] {
            let assets = Assets::from_html(&index(prefix)).unwrap();
            assert_eq!(assets.js, "client.js");
            assert_eq!(assets.wasm, "client_bg.wasm");
            assert_eq!(
                assets.manifest_url,
                format!("{prefix}offline-manifest.json")
            );
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&assets.manifest()).unwrap(),
                json!({"version": 1, "js": "client.js", "wasm": "client_bg.wasm"})
            );
        }
    }

    #[test]
    fn decodes_and_safely_embeds_manifest_url() {
        let html = index("/São&amp;Paulo/&lt;/");
        let assets = Assets::from_html(&html).unwrap();
        assert_eq!(assets.manifest_url, "/São&Paulo/</offline-manifest.json");
        let output = assets
            .install_manifest(&html, "export const x = 1;")
            .unwrap();
        assert!(output.contains(r#"content="/São&amp;Paulo/&lt;/offline-manifest.json""#));
        assert!(!output.contains("<script"));
        assert!(output.ends_with("\n</body></html>"));
    }

    #[test]
    fn rejects_missing_ambiguous_or_unbundled_runtime() {
        assert!(Assets::from_html("<body></body>").is_err());
        assert!(Assets::from_html(&format!("{}{}", index("/"), index("/"))).is_err());
        let assets = Assets::from_html(&index("/")).unwrap();
        assert!(
            assets
                .install_manifest(&index("/"), "\n import x from './snippet.js';")
                .is_err()
        );
        assert!(assets.install_manifest("<body>", "").is_err());
    }
}
