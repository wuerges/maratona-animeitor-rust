//! Browser I/O through web-sys; no handwritten JavaScript exporter.
use super::{
    export::{self, ExportedFile, Manifest, Resource, Resources, Stylesheet},
    OfflineMedia, OfflineSnapshot,
};
use futures::FutureExt;
use js_sys::{Array, Uint8Array};
use std::rc::Rc;
use url::Url;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Blob, BlobPropertyBag, Document, HtmlAnchorElement, HtmlLinkElement, HtmlStyleElement,
    RequestInit, Response,
};

fn js_error(value: JsValue) -> String {
    value
        .as_string()
        .or_else(|| {
            js_sys::Reflect::get(&value, &"message".into())
                .ok()
                .and_then(|v| v.as_string())
        })
        .unwrap_or_else(|| "Browser operation failed".into())
}

pub(super) async fn fetch_resource(url: String) -> Result<Resource, String> {
    let options = RequestInit::new();
    options.set_signal(Some(&web_sys::AbortSignal::timeout_with_u32(30_000)));
    let window = web_sys::window().ok_or("No browser window")?;
    let response: Response = JsFuture::from(window.fetch_with_str_and_init(&url, &options))
        .await
        .map_err(js_error)?
        .dyn_into()
        .map_err(js_error)?;
    if !response.ok() {
        return Err(format!("HTTP {}", response.status()));
    }
    let content_type = response
        .headers()
        .get("content-type")
        .map_err(js_error)?
        .unwrap_or_else(|| "application/octet-stream".into())
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let buffer = JsFuture::from(response.array_buffer().map_err(js_error)?)
        .await
        .map_err(js_error)?;
    Ok(Resource {
        bytes: Uint8Array::new(&buffer).to_vec(),
        content_type,
    })
}

pub(super) async fn collect_styles(
    document: &Document,
    resources: &Resources,
    base: &Url,
) -> Result<Vec<Stylesheet>, String> {
    let elements = document
        .query_selector_all("link[rel=\"stylesheet\"], style")
        .map_err(js_error)?;
    let mut styles = vec![];
    for i in 0..elements.length() {
        let node = elements.item(i).ok_or("Stylesheet disappeared")?;
        let style = if let Some(link) = node.dyn_ref::<HtmlLinkElement>() {
            if link.disabled() {
                continue;
            }
            let url = base.join(&link.href()).map_err(|e| e.to_string())?;
            Stylesheet {
                css: resources.stylesheet(url, vec![]).await,
                media: link.media(),
            }
        } else if let Some(style) = node.dyn_ref::<HtmlStyleElement>() {
            if style.disabled() {
                continue;
            }
            Stylesheet {
                css: resources
                    .rewrite_css(&style.text_content().unwrap_or_default(), base, vec![])
                    .await,
                media: style.media(),
            }
        } else {
            continue;
        };
        styles.push(style);
    }
    Ok(styles)
}

async fn prepare(
    mut snapshot: OfflineSnapshot,
    sources: OfflineMedia,
    progress: Rc<dyn Fn(String)>,
) -> Result<ExportedFile, String> {
    let window = web_sys::window().ok_or("No browser window")?;
    let document = window.document().ok_or("No browser document")?;
    let base = Url::parse(
        &document
            .base_uri()
            .map_err(js_error)?
            .ok_or("No document base URL")?,
    )
    .map_err(|e| e.to_string())?;
    let manifest_href = document
        .query_selector("meta[name=\"reveleitor-offline-manifest\"]")
        .map_err(js_error)?
        .and_then(|e| e.get_attribute("content"))
        .ok_or("Offline runtime manifest is missing; rebuild the client")?;
    let manifest_url = base.join(&manifest_href).map_err(|e| e.to_string())?;
    let resources = Resources::new(Rc::new(|url| fetch_resource(url).boxed_local()), progress);
    let manifest: Manifest =
        serde_json::from_slice(&resources.required(&manifest_url).await?.bytes)
            .map_err(|e| e.to_string())?;
    if manifest.version != 1 {
        return Err("Unsupported runtime manifest".into());
    }
    let js_url = manifest_url.join(&manifest.js).map_err(|e| e.to_string())?;
    let wasm_url = manifest_url
        .join(&manifest.wasm)
        .map_err(|e| e.to_string())?;
    let (js, wasm) =
        futures::try_join!(resources.required(&js_url), resources.required(&wasm_url))?;
    let runtime = String::from_utf8(js.bytes).map_err(|e| e.to_string())?;
    let styles = collect_styles(&document, &resources, &base).await?;
    snapshot.media = resources.media(sources, &base).await;
    snapshot.omitted_assets = resources.omissions();
    if let Ok(location) = Url::parse(&window.location().href().map_err(js_error)?) {
        if let Some((_, color)) = location
            .query_pairs()
            .find(|(key, _)| key == "background-color")
        {
            snapshot.settings.background_color = Some(color.into_owned());
        }
    }
    export::assemble(&snapshot, &runtime, &wasm.bytes, &styles)
}

pub(super) fn download(file: &ExportedFile) -> Result<(), String> {
    let options = BlobPropertyBag::new();
    options.set_type("text/html;charset=utf-8");
    let parts = Array::new();
    parts.push(&JsValue::from_str(&file.html));
    let blob = Blob::new_with_str_sequence_and_options(&parts, &options).map_err(js_error)?;
    let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(js_error)?;
    let result = (|| {
        let document = web_sys::window()
            .and_then(|w| w.document())
            .ok_or("No browser document")?;
        let anchor: HtmlAnchorElement = document
            .create_element("a")
            .map_err(js_error)?
            .dyn_into()
            .map_err(|_| "Could not create download anchor")?;
        anchor.set_href(&url);
        anchor.set_download(&file.filename);
        anchor.click();
        Ok(())
    })();
    // Keep the Blob URL alive until the browser has consumed the download.
    gloo_timers::callback::Timeout::new(60_000, move || {
        let _ = web_sys::Url::revoke_object_url(&url);
    })
    .forget();
    result
}

pub(super) async fn save(
    snapshot: OfflineSnapshot,
    sources: OfflineMedia,
    progress: Rc<dyn Fn(String)>,
) -> Result<String, String> {
    let file = prepare(snapshot, sources, progress).await?;
    download(&file)?;
    Ok(file.message)
}

#[cfg(all(test, target_family = "wasm"))]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn fetch_and_stylesheet_dom_use_browser_apis() {
        let resource = fetch_resource("data:text/plain;base64,U8OjbyBQYXVsbw==".into())
            .await
            .unwrap();
        assert_eq!(String::from_utf8(resource.bytes).unwrap(), "São Paulo");
        let document = web_sys::window().unwrap().document().unwrap();
        let style: HtmlStyleElement = document
            .create_element("style")
            .unwrap()
            .dyn_into()
            .unwrap();
        style.set_media("print");
        style.set_text_content(Some(
            ".offline-browser-test {background:url(data:image/png;base64,AA==)}",
        ));
        document.head().unwrap().append_child(&style).unwrap();
        let resources = Resources::new(
            Rc::new(|url| fetch_resource(url).boxed_local()),
            Rc::new(|_| ()),
        );
        let base = Url::parse(&document.base_uri().unwrap().unwrap()).unwrap();
        let styles = collect_styles(&document, &resources, &base).await.unwrap();
        style.remove();
        let exported = styles
            .iter()
            .find(|s| s.css.contains("offline-browser-test"))
            .unwrap();
        assert_eq!(exported.media, "print");
        assert!(exported.css.contains("data:image/png;base64,AA=="));
        assert!(resources.omissions().is_empty());
    }

    #[wasm_bindgen_test]
    async fn blob_payload_is_utf8_and_fetchable_without_a_server() {
        let parts = Array::new();
        parts.push(&JsValue::from_str("São Paulo </script>"));
        let blob = Blob::new_with_str_sequence(&parts).unwrap();
        let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
        let resource = fetch_resource(url.clone()).await.unwrap();
        web_sys::Url::revoke_object_url(&url).unwrap();
        assert_eq!(
            String::from_utf8(resource.bytes).unwrap(),
            "São Paulo </script>"
        );
    }
}
