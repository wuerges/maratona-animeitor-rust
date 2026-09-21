//! Real-browser regression test, driven entirely from Rust over WebDriver.
//! See README.md for the driver, browser and Trunk output environment variables.
use axum::{
    Router,
    extract::{State, WebSocketUpgrade, ws::Message},
    http::{HeaderMap, StatusCode, Uri, header::CONTENT_TYPE},
    response::IntoResponse,
    routing::get,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    env,
    error::Error,
    fs,
    net::TcpListener,
    path::PathBuf,
    process::{Child, Command},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::time::sleep;

type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
fn ensure(condition: bool, message: impl Into<String>) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(message.into().into())
    }
}

#[derive(Clone)]
struct Fixture {
    dist: PathBuf,
    port: u16,
    missing: Arc<AtomicBool>,
    core_failure: Arc<AtomicBool>,
    export_delay: Arc<AtomicBool>,
    audio_failure: Arc<AtomicBool>,
    requests: Arc<Mutex<HashMap<String, usize>>>,
}
impl Fixture {
    fn count(&self, path: &str) -> usize {
        *self.requests.lock().unwrap().get(path).unwrap_or(&0)
    }
}
const SVG: &str = "<svg xmlns='http://www.w3.org/2000/svg' width='30' height='30'><rect width='30' height='30' fill='red'/></svg>";
fn wav() -> Vec<u8> {
    let mut b = vec![0; 16044];
    b[0..4].copy_from_slice(b"RIFF");
    b[4..8].copy_from_slice(&16036u32.to_le_bytes());
    b[8..16].copy_from_slice(b"WAVEfmt ");
    b[16..20].copy_from_slice(&16u32.to_le_bytes());
    b[20..22].copy_from_slice(&1u16.to_le_bytes());
    b[22..24].copy_from_slice(&1u16.to_le_bytes());
    b[24..28].copy_from_slice(&8000u32.to_le_bytes());
    b[28..32].copy_from_slice(&16000u32.to_le_bytes());
    b[32..34].copy_from_slice(&2u16.to_le_bytes());
    b[34..36].copy_from_slice(&16u16.to_le_bytes());
    b[36..40].copy_from_slice(b"data");
    b[40..44].copy_from_slice(&16000u32.to_le_bytes());
    b
}
async fn socket(
    State(state): State<Fixture>,
    uri: Uri,
    upgrade: WebSocketUpgrade,
) -> impl IntoResponse {
    let path = uri.path().to_owned();
    *state
        .requests
        .lock()
        .unwrap()
        .entry(path.clone())
        .or_default() += 1;
    upgrade.on_upgrade(move |mut ws| async move {
        if path.ends_with("/timer") {
            let _ = ws
                .send(Message::Text(
                    json!({"current_time_seconds":300,"score_freeze_time_seconds":240})
                        .to_string()
                        .into(),
                ))
                .await;
        }
        while ws.recv().await.is_some() {}
    })
}
async fn serve(State(state): State<Fixture>, uri: Uri, headers: HeaderMap) -> impl IntoResponse {
    let path = uri.path();
    *state
        .requests
        .lock()
        .unwrap()
        .entry(path.to_owned())
        .or_default() += 1;
    let response =
        |mime: &str, bytes: Vec<u8>| (StatusCode::OK, [(CONTENT_TYPE, mime.to_owned())], bytes);
    let text = |mime: &str, text: String| response(mime, text.into_bytes());
    let json_response = |data: Value| text("application/json", json!({"data":data}).to_string());
    let missing = state.missing.load(Ordering::Relaxed);
    if path.ends_with(".wasm") && state.export_delay.load(Ordering::Relaxed) {
        sleep(Duration::from_secs(2)).await;
    }
    match path {
        "/config.json" => return text("application/json", "{}".into()),
        "/user-styles.css" => {
            return text(
                "text/css",
                "@import '/theme/nested.css' screen; .titulo {border-top: 3px solid rgb(1, 2, 3)}"
                    .into(),
            );
        }
        "/theme/nested.css" => {
            return text(
                "text/css",
                format!(
                    ".runstable {{ --offline-test: preserved; background-image:url(./pattern.svg),url(./pattern.svg) }}{}",
                    if missing {
                        format!(
                            ".titulo {{background-image:url(http://localhost:{}/theme/external.svg)}}",
                            state.port
                        )
                    } else {
                        String::new()
                    }
                ),
            );
        }
        "/theme/pattern.svg" | "/theme/external.svg" => return text("image/svg+xml", SVG.into()),
        _ => (),
    }
    if path.ends_with("/config") {
        return json_response(
            json!({"name":"Test", "codes":[".*"], "style":null,"ouro":1,"prata":2,"bronze":3,"sites":[{"name":"Site A","codes":["^a"]}],"photo_url_format":null,"sound_url_format":null}),
        );
    }
    if path.ends_with("/contest") {
        let teams: Vec<_> = ["a1","a2","b1"].into_iter().map(|login| json!({"login":login,"escola":"School","nome":format!("Team {login} São Paulo </script>")})).collect();
        return json_response(
            json!({"event":"test","contest":"Test","problems":["A"],"teams":teams,"time_seconds":300,"score_freeze_time_seconds":240,"penalty_seconds":1200}),
        );
    }
    if path.ends_with("/runs_secret") {
        assert_eq!(headers.get("authorization").unwrap(), "Bearer SECRET-TEST");
        return json_response(json!({"runs":[
            {"id":1,"team_login":"a1","prob":"A","time_seconds":100,"answer":"Y"},
            {"id":2,"team_login":"a2","prob":"A","time_seconds":250,"answer":"Y"},
            {"id":3,"team_login":"b1","prob":"A","time_seconds":260,"answer":"N"}
        ]}));
    }
    let fail = (missing && path == "/photos/a2.webp")
        || (state.audio_failure.load(Ordering::Relaxed)
            && matches!(path, "/sounds/a1.mp3" | "/sounds/a2.mp3"))
        || (state.core_failure.load(Ordering::Relaxed) && path.ends_with(".wasm"));
    if !fail {
        if path.starts_with("/photos/") {
            return text("image/svg+xml", SVG.into());
        }
        if path.starts_with("/sounds/") {
            return response("audio/wav", wav());
        }
        let file = if path == "/animeitor/test/main" {
            "index.html"
        } else {
            path.trim_start_matches("/animeitor/")
        };
        if !file.contains("..") && !file.starts_with('/') {
            if let Ok(bytes) = fs::read(state.dist.join(file)) {
                return response(
                    mime_guess::from_path(file).first_or_octet_stream().as_ref(),
                    bytes,
                );
            }
        }
    }
    (
        StatusCode::NOT_FOUND,
        [(CONTENT_TYPE, "text/plain".into())],
        b"missing".to_vec(),
    )
}

struct Driver {
    client: reqwest::Client,
    base: String,
    session: String,
    child: Child,
    browser: String,
    binary: String,
    downloads: PathBuf,
}
impl Drop for Driver {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
impl Driver {
    async fn start(downloads: PathBuf) -> Result<Self> {
        let browser = env::var("OFFLINE_BROWSER").unwrap_or_else(|_| "chrome".into());
        ensure(
            ["chrome", "firefox"].contains(&browser.as_str()),
            "OFFLINE_BROWSER must be chrome or firefox",
        )?;
        let driver = env::var(if browser == "chrome" {
            "CHROMEDRIVER"
        } else {
            "GECKODRIVER"
        })?;
        let binary = env::var("OFFLINE_BROWSER_BINARY")?;
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        drop(listener);
        let log = fs::File::create(downloads.join("webdriver.log"))?;
        let child = Command::new(driver)
            .arg(format!("--port={port}"))
            .stdout(log.try_clone()?)
            .stderr(log)
            .spawn()?;
        let mut driver = Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()?,
            base: format!("http://127.0.0.1:{port}"),
            session: String::new(),
            child,
            browser,
            binary,
            downloads,
        };
        let start = Instant::now();
        loop {
            if driver
                .client
                .get(format!("{}/status", driver.base))
                .send()
                .await
                .is_ok()
            {
                break;
            }
            ensure(
                start.elapsed() < Duration::from_secs(15),
                "WebDriver did not start",
            )?;
            sleep(Duration::from_millis(100)).await;
        }
        driver.new_session(false).await?;
        Ok(driver)
    }
    async fn request(&self, method: reqwest::Method, path: &str, body: Value) -> Result<Value> {
        let response: Value = self
            .client
            .request(method, format!("{}{path}", self.base))
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
        if response["value"].get("error").is_some() {
            return Err(response.to_string().into());
        }
        Ok(response["value"].clone())
    }
    async fn command(&self, method: reqwest::Method, path: &str, body: Value) -> Result<Value> {
        self.request(method, &format!("/session/{}{path}", self.session), body)
            .await
    }
    async fn new_session(&mut self, offline: bool) -> Result<()> {
        let mut caps = if self.browser == "chrome" {
            json!({
                "browserName":"chrome", "goog:chromeOptions":{"binary":self.binary,"args":["--headless=new","--no-sandbox","--disable-dev-shm-usage"],"prefs":{"download.default_directory":self.downloads,"download.prompt_for_download":false}},
                "goog:loggingPrefs":{"performance":"ALL","browser":"ALL"}
            })
        } else {
            json!({
                "browserName":"firefox", "moz:firefoxOptions":{"binary":self.binary,"args":["-headless"],"prefs":{"browser.download.folderList":2,"browser.download.dir":self.downloads,"browser.download.useDownloadDir":true,"browser.helperApps.neverAsk.saveToDisk":"text/html","browser.download.always_ask_before_handling_new_types":false,"media.autoplay.default":0}}
            })
        };
        if offline {
            caps["proxy"] = json!({"proxyType":"manual","httpProxy":"127.0.0.1:9","sslProxy":"127.0.0.1:9","noProxy":[]});
        }
        let response = self
            .request(
                reqwest::Method::POST,
                "/session",
                json!({"capabilities":{"alwaysMatch":caps}}),
            )
            .await?;
        self.session = response["sessionId"]
            .as_str()
            .ok_or("No WebDriver session")?
            .into();
        if offline && self.browser == "chrome" {
            self.command(
                reqwest::Method::POST,
                "/goog/cdp/execute",
                json!({"cmd":"Network.enable","params":{}}),
            )
            .await?;
            self.command(reqwest::Method::POST,"/goog/cdp/execute",json!({"cmd":"Network.emulateNetworkConditions","params":{"offline":true,"latency":0,"downloadThroughput":0,"uploadThroughput":0}})).await?;
        }
        Ok(())
    }
    async fn close_session(&self) {
        let _ = self.command(reqwest::Method::DELETE, "", json!({})).await;
    }
    async fn goto(&self, url: &str) -> Result<()> {
        self.command(reqwest::Method::POST, "/url", json!({"url":url}))
            .await?;
        Ok(())
    }
    async fn find(&self, using: &str, value: &str) -> Result<String> {
        let v = self
            .command(
                reqwest::Method::POST,
                "/element",
                json!({"using":using,"value":value}),
            )
            .await?;
        Ok(v["element-6066-11e4-a52e-4f735466cecf"]
            .as_str()
            .ok_or("No element")?
            .into())
    }
    async fn property(&self, selector: &str, endpoint: &str) -> Result<Value> {
        let element = self.find("css selector", selector).await?;
        self.command(
            reqwest::Method::GET,
            &format!("/element/{element}/{endpoint}"),
            json!({}),
        )
        .await
    }
    async fn click(&self, using: &str, selector: &str) -> Result<()> {
        let start = Instant::now();
        loop {
            if let Ok(element) = self.find(using, selector).await {
                if self
                    .command(
                        reqwest::Method::POST,
                        &format!("/element/{element}/click"),
                        json!({}),
                    )
                    .await
                    .is_ok()
                {
                    return Ok(());
                }
            }
            ensure(
                start.elapsed() < Duration::from_secs(20),
                format!("Could not click {selector}"),
            )?;
            sleep(Duration::from_millis(100)).await;
        }
    }
    async fn button(&self, label: &str) -> Result<()> {
        self.click("xpath", &format!("//button[normalize-space(.)='{label}']"))
            .await
    }
    async fn settings(&self) -> Result<()> {
        self.click("css selector", ".settings-accordion > summary")
            .await
    }
    async fn set_text(&self, selector: &str, text: &str) -> Result<()> {
        let element = self.find("css selector", selector).await?;
        self.command(
            reqwest::Method::POST,
            &format!("/element/{element}/clear"),
            json!({}),
        )
        .await?;
        self.command(
            reqwest::Method::POST,
            &format!("/element/{element}/value"),
            json!({"text":text}),
        )
        .await?;
        Ok(())
    }
    async fn wait_text(&self, selector: &str, expected: &str) -> Result<String> {
        let start = Instant::now();
        loop {
            let text = self
                .property(selector, "text")
                .await
                .unwrap_or_default()
                .as_str()
                .unwrap_or_default()
                .to_owned();
            if text.contains(expected) {
                return Ok(text);
            }
            ensure(
                start.elapsed() < Duration::from_secs(20),
                format!("Expected {expected:?} in {selector}; found {text:?}"),
            )?;
            sleep(Duration::from_millis(100)).await;
        }
    }
    async fn board(&self) -> Result<Value> {
        let mut rows = vec![];
        for login in ["a1", "a2"] {
            let selector = format!("#{login}");
            rows.push(json!({"id":login,"text":self.property(&selector,"text").await?,"style":self.property(&selector,"attribute/style").await?}));
        }
        Ok(json!(rows))
    }
    async fn key(&self, key: &str) -> Result<()> {
        self.command(reqwest::Method::POST,"/actions",json!({"actions":[{"type":"key","id":"keyboard","actions":[{"type":"keyDown","value":key},{"type":"keyUp","value":key}]}]})).await?;
        Ok(())
    }
}

async fn run(driver: &mut Driver, fixture: &Fixture) -> Result<()> {
    let base = format!("http://127.0.0.1:{}/animeitor/test/main", fixture.port);
    for query in ["", "?settings=false", "?settings=true"] {
        driver.goto(&format!("{base}{query}")).await?;
        driver.wait_text(".contest-container", "Team a1").await?;
        let visible = driver
            .find("css selector", ".settings-accordion")
            .await
            .is_ok();
        ensure(
            visible == query.ends_with("true"),
            "Scoreboard settings query gate changed",
        )?;
        if visible {
            ensure(
                driver
                    .property(".settings-accordion", "property/open")
                    .await?
                    == false,
                "Settings must start collapsed",
            )?;
            driver.settings().await?;
            ensure(
                driver.find("css selector", ".offline-save").await.is_err(),
                "Public scoreboard must not offer a reveal export",
            )?;
        }
    }
    driver.goto(&format!("http://127.0.0.1:{}/animeitor/test/main?secret=SECRET-TEST&sede=Site%20A&background-color=%23123456",fixture.port)).await?;
    driver.button("OK").await?;
    let frozen = driver.board().await?;
    ensure(
        driver.property("#b1", "css/display").await? == "none",
        "Other sede should be hidden",
    )?;
    driver.button("All").await?;
    let final_scores = driver.board().await?;
    fixture.audio_failure.store(true, Ordering::Relaxed);
    driver.settings().await?;
    driver.click("css selector", "#settings-autoplay").await?;
    driver.settings().await?;
    for team in ["a1", "a2", "a1"] {
        if team == "a2" {
            driver.button("Reset").await?;
        }
        driver.click("css selector", &format!("#{team}")).await?;
        if team == "a2" {
            ensure(
                driver.find("css selector", "audio").await.is_err(),
                "Pending team should not play audio",
            )?;
            driver.button("All").await?;
            // All clears the focused team and closes its photo. Reopen the
            // resolved team before checking its fallback audio.
            ensure(
                driver.find("css selector", "#foto_a2").await.is_err(),
                "All should close the team photo",
            )?;
            driver.click("css selector", "#a2").await?;
        }
        for change_setting in [false, true] {
            if change_setting {
                driver.settings().await?;
                driver
                    .click("css selector", "#settings-team-details")
                    .await?;
                driver.settings().await?;
            }
            let start = Instant::now();
            loop {
                let source = driver
                    .property("audio", "property/currentSrc")
                    .await
                    .unwrap_or(Value::Null);
                let playing = driver
                    .property("audio", "property/paused")
                    .await
                    .unwrap_or(Value::Null)
                    == false;
                if source
                    .as_str()
                    .is_some_and(|s| s.ends_with("/applause.mp3"))
                    && playing
                {
                    break;
                }
                ensure(
                    start.elapsed() < Duration::from_secs(10),
                    format!(
                        "Fallback audio failed for {team} (settings changed: {change_setting}, source: {source}, playing: {playing})"
                    ),
                )?;
                sleep(Duration::from_millis(100)).await;
            }
        }
        driver.click("css selector", ".foto_img").await?;
    }
    driver.settings().await?;
    driver.click("css selector", "#settings-autoplay").await?;
    driver.settings().await?;
    fixture.audio_failure.store(false, Ordering::Relaxed);
    ensure(
        final_scores != frozen,
        "All must change the frozen standings",
    )?;
    ensure(
        driver
            .property(".settings-accordion", "property/open")
            .await?
            == false,
        "Reveleitor settings must start collapsed",
    )?;
    ensure(
        driver
            .property(".settings-accordion", "css/position")
            .await?
            == "fixed",
        "Settings must be fixed",
    )?;
    ensure(
        driver.property(".settings-accordion", "css/top").await? == "8px",
        "Settings top offset changed",
    )?;
    ensure(
        driver.property(".settings-accordion", "css/right").await? == "8px",
        "Settings must sit at the right edge",
    )?;
    let board_rect = driver.property(".revelationpanel", "rect").await?;
    driver.settings().await?;
    ensure(
        driver.property(".revelationpanel", "rect").await? == board_rect,
        "Opening settings shifted the standings",
    )?;
    let window_rect = driver
        .command(reqwest::Method::GET, "/window/rect", json!({}))
        .await?;
    driver
        .command(
            reqwest::Method::POST,
            "/window/rect",
            json!({"width":390,"height":480}),
        )
        .await?;
    let panel = driver
        .property(".settings-accordion-content", "rect")
        .await?;
    let width = driver
        .property("html", "property/clientWidth")
        .await?
        .as_f64()
        .unwrap();
    let height = driver
        .property("html", "property/clientHeight")
        .await?
        .as_f64()
        .unwrap();
    ensure(
        panel["x"].as_f64().unwrap() >= 0.0
            && panel["x"].as_f64().unwrap() + panel["width"].as_f64().unwrap() <= width,
        "Settings exceeds narrow viewport width",
    )?;
    ensure(
        panel["y"].as_f64().unwrap() + panel["height"].as_f64().unwrap() <= height,
        "Settings exceeds short viewport height",
    )?;
    let screenshot = driver
        .command(reqwest::Method::GET, "/screenshot", json!({}))
        .await?;
    fs::write(
        driver.downloads.join("settings-narrow.png"),
        STANDARD.decode(screenshot.as_str().unwrap())?,
    )?;
    driver
        .command(reqwest::Method::POST, "/window/rect", window_rect)
        .await?;
    driver.set_text("#settings-team-background", "my").await?;
    for key in ["\u{e014}", "\u{e012}", "\u{e013}", "\u{e015}", "\u{e003}"] {
        driver.key(key).await?;
    }
    ensure(
        driver.board().await? == final_scores,
        "Settings keyboard input changed revelation state",
    )?;
    driver
        .set_text("#settings-team-background", "black")
        .await?;
    driver.click("css selector", "#settings-mute").await?;
    ensure(
        driver
            .property("#settings-mute", "property/checked")
            .await?
            == true,
        "Settings edits did not apply",
    )?;
    driver.key("\u{e00c}").await?;
    ensure(
        driver
            .property(".settings-accordion", "property/open")
            .await?
            == false,
        "Escape must collapse settings",
    )?;
    let active = driver
        .command(reqwest::Method::GET, "/element/active", json!({}))
        .await?;
    ensure(
        active["element-6066-11e4-a52e-4f735466cecf"]
            == driver
                .find("css selector", ".settings-accordion > summary")
                .await?,
        "Escape must return focus to Settings",
    )?;
    driver.key("\u{e007}").await?;
    ensure(
        driver
            .property(".settings-accordion", "property/open")
            .await?
            == true,
        "Enter must open the native disclosure",
    )?;
    ensure(
        driver
            .property("#settings-mute", "property/checked")
            .await?
            == true,
        "Collapsing settings lost form state",
    )?;
    driver.click("css selector", "#settings-mute").await?;
    let save_id = driver.find("css selector", ".offline-save button").await?;
    // Save explicit per-team preferences, including the latest volume edit.
    driver
        .command(
            reqwest::Method::POST,
            "/window/rect",
            json!({"width":1200,"height":1000}),
        )
        .await?;
    driver.settings().await?;
    driver.click("css selector", "#a1").await?;
    driver
        .click("css selector", ".volume_controls input[type=checkbox]")
        .await?;
    driver
        .click("css selector", ".volume_controls input[type=range]")
        .await?;
    let saved_volume = driver
        .property(".volume_controls input[type=range]", "property/value")
        .await?;
    ensure(
        saved_volume != "100",
        "Fixture must save a nondefault team volume",
    )?;
    driver.click("css selector", ".foto_img").await?;
    driver.click("css selector", "#a2").await?;
    for _ in 0..2 {
        driver
            .click("css selector", ".volume_controls input[type=checkbox]")
            .await?;
    }
    driver.click("css selector", ".foto_img").await?;
    driver.settings().await?;
    fixture.core_failure.store(true, Ordering::Relaxed);
    driver.button("Save offline HTML").await?;
    driver
        .wait_text("#offline-save-status", "Could not save offline file")
        .await?;
    ensure(
        !driver
            .downloads
            .join("Test-Site A-reveleitor.html")
            .exists(),
        "Failed exports must not download",
    )?;
    fixture.core_failure.store(false, Ordering::Relaxed);
    fixture.export_delay.store(true, Ordering::Relaxed);
    let previous_patterns = fixture.count("/theme/pattern.svg");
    driver.button("Save offline HTML").await?;
    ensure(
        driver
            .property(".offline-save button", "property/disabled")
            .await?
            == true,
        "Save should be busy",
    )?;
    driver.settings().await?;
    ensure(
        driver
            .property(".settings-accordion", "property/open")
            .await?
            == false,
        "Settings did not collapse during export",
    )?;
    driver.settings().await?;
    ensure(
        driver.find("css selector", ".offline-save button").await? == save_id,
        "Closing settings remounted the save interface",
    )?;
    driver
        .wait_text(
            "#offline-save-status",
            "Offline file saved. 2 unavailable assets",
        )
        .await?;
    fixture.export_delay.store(false, Ordering::Relaxed);
    ensure(
        fixture.count("/theme/pattern.svg") - previous_patterns == 1,
        "Duplicate CSS assets were downloaded more than once",
    )?;
    let downloaded = driver.downloads.join("Test-Site A-reveleitor.html");
    let start = Instant::now();
    while !downloaded.exists() {
        ensure(
            start.elapsed() < Duration::from_secs(20),
            "Download did not finish",
        )?;
        sleep(Duration::from_millis(100)).await;
    }
    let saved = driver.downloads.join("São Paulo offline.html");
    fs::rename(downloaded, &saved)?;
    let html = fs::read_to_string(&saved)?;
    let encoded = html
        .split("type=\"application/octet-stream\">")
        .nth(1)
        .ok_or("No payload")?
        .split('<')
        .next()
        .unwrap();
    let payload: Value = serde_json::from_slice(&STANDARD.decode(encoded)?)?;
    ensure(payload["version"] == 1, "Wrong snapshot version")?;
    ensure(
        payload["settings"]["team_settings"]["a1"]["autoplay"] == true,
        "Enabled team autoplay was not saved",
    )?;
    ensure(
        payload["settings"]["team_settings"]["a2"]["autoplay"] == false,
        "Disabled team autoplay was not saved",
    )?;
    ensure(
        payload["settings"]["team_settings"]["a1"]["volume"].to_string()
            == saved_volume.as_str().unwrap(),
        "Latest team volume was not saved",
    )?;
    ensure(
        payload["contest"]["teams"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>()
            == ["a1", "a2"],
        "Wrong sede teams",
    )?;
    ensure(
        payload["runs"]["runs"].as_object().unwrap().len() == 2,
        "Wrong runs",
    )?;
    ensure(
        !payload.to_string().contains("SECRET-TEST"),
        "Snapshot contains credentials",
    )?;
    ensure(
        payload["omitted_assets"].as_array().unwrap().len() == 2,
        "Wrong omissions",
    )?;
    ensure(
        payload["media"]["photos"]["fake"]
            .as_str()
            .unwrap()
            .starts_with("data:image/"),
        "Missing fallback photo",
    )?;
    ensure(
        payload["media"]["sounds"]["applause"]
            .as_str()
            .unwrap()
            .starts_with("data:audio/"),
        "Missing fallback sound",
    )?;
    fixture.missing.store(false, Ordering::Relaxed);
    driver.button("Save offline HTML").await?;
    driver
        .wait_text("#offline-save-status", "Offline file saved. Double-click")
        .await?;
    driver.close_session().await;
    driver.new_session(true).await?;
    let before_requests = fixture.requests.lock().unwrap().clone();
    let url = url::Url::from_file_path(&saved).unwrap().to_string();
    driver.goto(&url).await?;
    driver.button("OK").await?;
    ensure(
        driver
            .property(".settings-accordion", "property/open")
            .await?
            == false,
        "Offline settings must start collapsed",
    )?;
    driver.settings().await?;
    ensure(
        driver
            .find("css selector", "#settings-secret")
            .await
            .is_err(),
        "Offline settings must not show credentials",
    )?;
    ensure(
        driver.find("css selector", ".offline-save").await.is_err(),
        "Offline settings must not show export controls",
    )?;
    driver.set_text("#settings-background", "#abcdef").await?;
    ensure(
        driver
            .property("body", "css/background-color")
            .await?
            .as_str()
            .unwrap()
            .contains("171, 205, 239"),
        "Offline presentation settings did not apply",
    )?;
    driver.set_text("#settings-background", "#123456").await?;
    driver.settings().await?;
    ensure(
        driver.board().await? == frozen,
        "Offline copy did not start frozen",
    )?;
    ensure(
        driver.property("body", "css/background-color").await? == "rgba(18, 52, 86, 1)"
            || driver.property("body", "css/background-color").await? == "rgb(18, 52, 86)",
        "Background color was lost",
    )?;
    ensure(
        driver
            .property(".titulo", "css/border-top-color")
            .await?
            .as_str()
            .unwrap()
            .contains("1, 2, 3"),
        "Custom CSS was lost",
    )?;
    ensure(
        driver
            .property(".runstable", "css/background-image")
            .await?
            .as_str()
            .unwrap()
            .contains("data:image/svg+xml;base64,"),
        "Nested CSS assets were lost",
    )?;
    for label in [
        "→", "←", "↑", "↓", "Top 100", "Top 50", "Top 30", "Top 10", "All",
    ] {
        driver.button(label).await?;
    }
    ensure(
        driver.board().await? == final_scores,
        "Offline final standings differ",
    )?;
    for (team, autoplay) in [("a1", true), ("a2", false)] {
        driver.click("css selector", &format!("#{team}")).await?;
        ensure(
            driver
                .property(".volume_controls input[type=checkbox]", "property/checked")
                .await?
                == autoplay,
            "Saved team autoplay was not restored",
        )?;
        if team == "a1" {
            ensure(
                driver
                    .property(".volume_controls input[type=range]", "property/value")
                    .await?
                    == saved_volume,
                "Saved team volume was not restored",
            )?;
            ensure(
                driver.property("audio", "property/volume").await?
                    == saved_volume.as_str().unwrap().parse::<f64>()? / 100.0,
                "Restored volume did not reach the audio element",
            )?;
            // Continue the existing per-team toggle tests from autoplay off.
            driver
                .click("css selector", ".volume_controls input[type=checkbox]")
                .await?;
        }
        driver.click("css selector", ".foto_img").await?;
    }
    driver.button("Reset").await?;
    ensure(driver.board().await? == frozen, "Reset failed")?;
    for key in ["\u{e014}", "\u{e012}", "\u{e013}", "\u{e015}", "\u{e003}"] {
        driver.key(key).await?;
    }
    ensure(driver.board().await? == frozen, "Keyboard controls failed")?;
    driver.button("All").await?;
    driver.click("css selector", "#a2").await?;
    let start = Instant::now();
    loop {
        if driver
            .property(".foto_img", "property/naturalWidth")
            .await?
            .as_u64()
            .unwrap_or(0)
            > 0
        {
            break;
        }
        ensure(
            start.elapsed() < Duration::from_secs(10),
            "Fallback photo did not load",
        )?;
        sleep(Duration::from_millis(100)).await;
    }
    driver.key("m").await?;
    let start = Instant::now();
    loop {
        if driver
            .property("audio", "property/paused")
            .await
            .unwrap_or(Value::Null)
            == false
        {
            break;
        }
        ensure(
            start.elapsed() < Duration::from_secs(10),
            "Audio did not play after user interaction",
        )?;
        sleep(Duration::from_millis(100)).await;
    }
    driver.settings().await?;
    driver.set_text("#settings-team-background", "my").await?;
    ensure(
        driver.find("css selector", ".foto_img").await.is_ok(),
        "Typing Y in settings toggled the team photo",
    )?;
    ensure(
        driver
            .property(".volume_controls input[type=checkbox]", "property/checked")
            .await?
            == true,
        "Typing M in settings toggled team autoplay",
    )?;
    driver
        .set_text("#settings-team-background", "black")
        .await?;
    driver.key("\u{e00c}").await?;
    driver
        .click("css selector", ".volume_controls label")
        .await?;
    driver.key("y").await?;
    // M enabled only a2; a1 retains its explicit off setting.
    driver.click("css selector", "#a1").await?;
    ensure(
        driver
            .property(".volume_controls input[type=checkbox]", "property/checked")
            .await?
            == false,
        "A team's autoplay override affected another team",
    )?;
    driver.settings().await?;
    driver.click("css selector", "#settings-autoplay").await?;
    driver.settings().await?;
    ensure(
        driver
            .property(".volume_controls input[type=checkbox]", "property/checked")
            .await?
            == false,
        "Global autoplay overwrote an explicit team setting",
    )?;
    driver
        .click("css selector", ".volume_controls input[type=checkbox]")
        .await?;
    for team in ["a1", "a2", "a1"] {
        if team != "a1" || driver.find("css selector", "#foto_a1").await.is_err() {
            driver.click("css selector", ".foto_img").await?;
            driver.click("css selector", &format!("#{team}")).await?;
        }
        let start = Instant::now();
        loop {
            if driver
                .property("audio", "property/paused")
                .await
                .unwrap_or(Value::Null)
                == false
            {
                break;
            }
            ensure(
                start.elapsed() < Duration::from_secs(10),
                "Offline audio did not play for every team with autoplay enabled",
            )?;
            sleep(Duration::from_millis(100)).await;
        }
        ensure(
            driver
                .property("audio", "property/currentSrc")
                .await?
                .as_str()
                .unwrap()
                .starts_with("data:audio/"),
            "Team audio was not embedded",
        )?;
    }
    driver.click("css selector", ".foto_img").await?;
    let screenshot = driver
        .command(reqwest::Method::GET, "/screenshot", json!({}))
        .await?;
    fs::write(
        driver.downloads.join("offline.png"),
        STANDARD.decode(screenshot.as_str().unwrap())?,
    )?;
    // Local edits must override the embedded defaults on the next opening.
    driver
        .command(
            reqwest::Method::POST,
            "/window/rect",
            json!({"width":1200,"height":1000}),
        )
        .await?;
    driver.settings().await?;
    driver.set_text("#settings-background", "#654321").await?;
    driver.settings().await?;
    driver.click("css selector", "#a1").await?;
    driver
        .click("css selector", ".volume_controls input[type=checkbox]")
        .await?;
    driver
        .click("css selector", ".volume_controls input[type=range]")
        .await?;
    driver.key("\u{e010}").await?; // End sets volume to 100; this is the last settings edit.
    ensure(
        driver
            .property(".volume_controls input[type=range]", "property/value")
            .await?
            == "100",
        "Volume edit did not apply",
    )?;
    driver.goto(&url).await?;
    driver.button("OK").await?;
    ensure(
        driver.board().await? == frozen,
        "Reopening must restart frozen",
    )?;
    driver.settings().await?;
    ensure(
        driver
            .property("#settings-autoplay", "property/checked")
            .await?
            == true,
        "Stored global autoplay must override the file",
    )?;
    ensure(
        driver
            .property("#settings-background", "property/value")
            .await?
            == "#654321",
        "Stored background must override the file",
    )?;
    driver.settings().await?;
    for (team, autoplay) in [("a1", false), ("a2", true)] {
        driver.click("css selector", &format!("#{team}")).await?;
        ensure(
            driver
                .property(".volume_controls input[type=checkbox]", "property/checked")
                .await?
                == autoplay,
            "Stored team autoplay must override the file",
        )?;
        if team == "a1" {
            ensure(
                driver
                    .property(".volume_controls input[type=range]", "property/value")
                    .await?
                    == "100",
                "The last volume edit was not persisted",
            )?;
        }
        driver.click("css selector", ".foto_img").await?;
    }
    for (name, data, expected) in [
        (
            "unsupported",
            STANDARD.encode(r#"{"version":2}"#),
            "Unsupported offline file version",
        ),
        ("malformed", "!".into(), "Invalid offline file encoding"),
    ] {
        let invalid = driver.downloads.join(format!("{name}.html"));
        fs::write(&invalid, html.replace(encoded, &data))?;
        driver
            .goto(url::Url::from_file_path(invalid).unwrap().as_str())
            .await?;
        driver.wait_text("[role=alert]", expected).await?;
    }
    ensure(
        *fixture.requests.lock().unwrap() == before_requests,
        "Offline copy contacted the fixture server",
    )?;
    if driver.browser == "chrome" {
        let logs = driver
            .command(reqwest::Method::POST, "/log", json!({"type":"performance"}))
            .await?;
        for log in logs.as_array().unwrap() {
            let record: Value = serde_json::from_str(log["message"].as_str().unwrap())?;
            let message = &record["message"];
            if ["Network.requestWillBeSent", "Network.webSocketCreated"]
                .contains(&message["method"].as_str().unwrap_or_default())
            {
                let url = message["params"]["request"]["url"]
                    .as_str()
                    .or(message["params"]["url"].as_str())
                    .unwrap_or_default();
                ensure(
                    !["http:", "https:", "ws:", "wss:"]
                        .iter()
                        .any(|scheme| url.starts_with(scheme)),
                    format!("Offline network request: {url}"),
                )?;
            }
        }
        let logs = driver
            .command(reqwest::Method::POST, "/log", json!({"type":"browser"}))
            .await?;
        ensure(
            !logs
                .as_array()
                .unwrap()
                .iter()
                .any(|entry| entry["level"] == "SEVERE"),
            format!("Browser errors: {logs}"),
        )?;
    }
    Ok(())
}

#[tokio::test]
#[ignore = "requires a built client, WebDriver and browser; see README"]
async fn offline_html_in_real_browser() -> Result<()> {
    let dist = PathBuf::from(env::var("OFFLINE_CLIENT_DIST")?).canonicalize()?;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let fixture = Fixture {
        dist,
        port: listener.local_addr()?.port(),
        missing: Arc::new(AtomicBool::new(true)),
        core_failure: Arc::new(AtomicBool::new(false)),
        export_delay: Arc::new(AtomicBool::new(false)),
        audio_failure: Arc::new(AtomicBool::new(false)),
        requests: Default::default(),
    };
    let app = Router::new()
        .route("/api/events/{event}/timer", get(socket))
        .route(
            "/api/events/{event}/contests/{contest}/runs_ws",
            get(socket),
        )
        .route(
            "/api/events/{event}/contests/{contest}/remote_control/{key}",
            get(socket),
        )
        .fallback(serve)
        .with_state(fixture.clone());
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    let temp = tempfile::Builder::new()
        .prefix("reveleitor-rust-browser-")
        .tempdir()?
        .keep();
    eprintln!("Browser artifacts: {}", temp.display());
    let mut driver = Driver::start(temp).await?;
    let result = run(&mut driver, &fixture).await;
    if result.is_err() {
        if let Ok(screenshot) = driver
            .command(reqwest::Method::GET, "/screenshot", json!({}))
            .await
        {
            if let Some(encoded) = screenshot.as_str() {
                fs::write(
                    driver.downloads.join("failure.png"),
                    STANDARD.decode(encoded)?,
                )?;
            }
        }
    }
    driver.close_session().await;
    server.abort();
    result
}
