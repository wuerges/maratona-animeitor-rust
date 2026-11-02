//! Dedicated-target workload generator. Never targets production by default.
use clap::Parser;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio_tungstenite::{Connector, tungstenite::Message};

#[derive(Parser, Debug, Clone)]
#[command(about = "Measure two browser-style WebSockets per viewer against a dedicated target")]
struct Args {
    #[arg(long)]
    target: url::Url,
    #[arg(long)]
    event: String,
    #[arg(long)]
    contest: String,
    #[arg(long, default_value_t = 10)]
    viewers: usize,
    #[arg(long, default_value_t = 10)]
    ramp_seconds: u64,
    #[arg(long, default_value_t = 60)]
    hold_seconds: u64,
    #[arg(long, default_value_t = 1)]
    seed: u64,
    #[arg(long, default_value = "none", value_parser = ["none", "quarter", "all"])]
    reconnect: String,
    #[arg(long)]
    ca: Option<PathBuf>,
    #[arg(long)]
    expect_file: Option<PathBuf>,
    #[arg(long, default_value = "viewer-load-report.json")]
    output: PathBuf,
    #[arg(long)]
    trace: Option<PathBuf>,
    #[arg(long, default_value_t = 100)]
    sample_viewers: usize,
    #[arg(long, default_value_t = 100_000)]
    max_run_ids: usize,
}
#[derive(Default)]
struct Stats {
    active: AtomicU64,
    peak: AtomicU64,
    opened: AtomicU64,
    failed_attempts: AtomicU64,
    messages: AtomicU64,
    invalid: AtomicU64,
    mismatch: AtomicU64,
    validated: AtomicU64,
    replay_complete: AtomicU64,
    replay_max_ms: AtomicU64,
    unexpected_disconnects: AtomicU64,
    trace_dropped: AtomicU64,
}
struct Active(Arc<Stats>);
impl Active {
    fn new(stats: Arc<Stats>) -> Self {
        let n = stats.active.fetch_add(1, Ordering::Relaxed) + 1;
        stats.peak.fetch_max(n, Ordering::Relaxed);
        Self(stats)
    }
}
impl Drop for Active {
    fn drop(&mut self) {
        self.0.active.fetch_sub(1, Ordering::Relaxed);
    }
}
#[derive(Deserialize)]
struct Expected {
    #[serde(skip)]
    ids: std::collections::HashSet<i64>,
    runs: Vec<data::event::Run>,
    timer: Option<data::event::PublicTimer>,
}
#[derive(Serialize)]
struct Report {
    version: &'static str,
    target: String,
    event: String,
    contest: String,
    viewers: usize,
    ramp_seconds: u64,
    hold_seconds: u64,
    reconnect: String,
    seed: u64,
    elapsed_seconds: f64,
    peak_sockets: u64,
    opened_sockets: u64,
    failed_attempts: u64,
    messages: u64,
    invalid_messages: u64,
    final_mismatches: u64,
    validated_streams: u64,
    completed_initial_replays: u64,
    max_initial_replay_ms: u64,
    expected_state_checked: bool,
    unexpected_disconnects: u64,
    trace_dropped: u64,
    samples: Vec<Sample>,
    capacity_claim: &'static str,
}
#[derive(Serialize)]
struct Sample {
    elapsed_seconds: f64,
    active_sockets: u64,
    opened_sockets: u64,
    failed_attempts: u64,
}
fn unix_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}
fn trace_message(
    trace: &Option<tokio::sync::mpsc::Sender<String>>,
    stats: &Stats,
    viewer: usize,
    timer: bool,
    text: &str,
) {
    if let Some(trace) = trace {
        let value =
            serde_json::from_str::<serde_json::Value>(text).unwrap_or(serde_json::Value::Null);
        let line = serde_json::json!({"viewer":viewer,"kind":if timer {"timer"} else {"run"},"received_unix_ms":unix_ms(),"message":value}).to_string();
        if trace.try_send(line).is_err() {
            stats.trace_dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
}
fn fingerprint(run: &data::event::Run) -> u64 {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    serde_json::to_vec(run).unwrap().hash(&mut hash);
    hash.finish()
}
fn path(args: &Args, timer: bool) -> String {
    let mut url = args.target.clone();
    let scheme = if url.scheme() == "https" { "wss" } else { "ws" };
    url.set_scheme(scheme).unwrap();
    {
        let mut path = url.path_segments_mut().unwrap();
        path.pop_if_empty().extend(["api", "events", &args.event]);
        if timer {
            path.push("timer");
        } else {
            path.extend(["contests", &args.contest, "runs_ws"]);
        }
    }
    url.to_string()
}
fn jitter(seed: &mut u64, attempt: u32) -> Duration {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    let base = (1000u64 << attempt.min(5)).min(30_000);
    Duration::from_millis(base / 2 + *seed % (base / 2 + 1))
}
async fn expected(path: &Option<PathBuf>) -> Result<Option<Expected>, String> {
    match path {
        None => Ok(None),
        Some(path) => {
            let bytes = tokio::fs::read(path)
                .await
                .map_err(|e| format!("reading expectation: {e}"))?;
            serde_json::from_slice::<Expected>(&bytes)
                .map(|mut expected| {
                    expected.ids = expected.runs.iter().map(|r| r.id).collect();
                    Some(expected)
                })
                .map_err(|e| format!("parsing expectation: {e}"))
        }
    }
}
async fn stream(
    args: Arc<Args>,
    stats: Arc<Stats>,
    connector: Option<Connector>,
    (id, timer): (usize, bool),
    start: tokio::time::Instant,
    expectations: Arc<tokio::sync::RwLock<Option<Arc<Expected>>>>,
    trace: Option<tokio::sync::mpsc::Sender<String>>,
) -> Result<(), String> {
    let ramp = Duration::from_secs_f64(args.ramp_seconds as f64 * id as f64 / args.viewers as f64);
    tokio::time::sleep_until(start + ramp).await;
    let end = start + Duration::from_secs(args.ramp_seconds + args.hold_seconds);
    let disconnect = start + Duration::from_secs(args.ramp_seconds + args.hold_seconds / 2);
    let affected =
        args.reconnect == "all" || (args.reconnect == "quarter" && id < args.viewers.div_ceil(4));
    let mut dropped = !affected;
    let mut seed = args
        .seed
        .wrapping_add((id as u64) * 2 + u64::from(timer) + 1)
        .max(1);
    let mut attempt = 0;
    let mut latest = HashMap::new();
    let mut latest_timer = None;
    let initial_expected = expectations.read().await.clone();
    let expected_ids = initial_expected.as_ref().map(|value| &value.ids);
    let mut replay_reported = timer || expected_ids.is_none_or(|ids| ids.is_empty());
    let url = path(&args, timer);
    loop {
        if tokio::time::Instant::now() >= end {
            break;
        }
        let connecting =
            tokio_tungstenite::connect_async_tls_with_config(&url, None, false, connector.clone());
        let connection = tokio::select! { _ = tokio::time::sleep_until(end) => break, value = tokio::time::timeout(Duration::from_secs(15), connecting) => value };
        if let Ok(Ok((mut socket, _))) = connection {
            stats.opened.fetch_add(1, Ordering::Relaxed);
            let _active = Active::new(stats.clone());
            let opened = Instant::now();
            let mut intentional_close = false;
            loop {
                let next = tokio::select! {
                    _ = tokio::time::sleep_until(end) => { intentional_close = true; let _ = tokio::time::timeout(Duration::from_secs(5), socket.close(None)).await; break; },
                    _ = tokio::time::sleep_until(disconnect), if !dropped => { dropped = true; intentional_close = true; let _ = tokio::time::timeout(Duration::from_secs(5), socket.close(None)).await; break; },
                    next = socket.next() => next,
                };
                match next {
                    Some(Ok(Message::Text(text))) => {
                        stats.messages.fetch_add(1, Ordering::Relaxed);
                        if timer {
                            match serde_json::from_str::<data::event::PublicTimer>(&text) {
                                Ok(value) => {
                                    if latest_timer.is_some()
                                        && latest_timer != Some(value)
                                        && id < args.sample_viewers
                                    {
                                        trace_message(&trace, &stats, id, true, &text);
                                    }
                                    latest_timer = Some(value);
                                }
                                Err(_) => {
                                    stats.invalid.fetch_add(1, Ordering::Relaxed);
                                }
                            }
                        } else {
                            match serde_json::from_str::<data::event::Run>(&text) {
                                Ok(run) => {
                                    if !latest.contains_key(&run.id)
                                        && latest.len() >= args.max_run_ids
                                    {
                                        return Err(
                                            "max-run-ids exceeded; increase bound for the dataset"
                                                .into(),
                                        );
                                    }
                                    let hash = fingerprint(&run);
                                    if replay_reported
                                        && latest.get(&run.id) != Some(&hash)
                                        && id < args.sample_viewers
                                    {
                                        trace_message(&trace, &stats, id, false, &text);
                                    }
                                    latest.insert(run.id, hash);
                                    if !replay_reported
                                        && expected_ids.is_some_and(|ids| {
                                            latest.len() >= ids.len()
                                                && ids.iter().all(|id| latest.contains_key(id))
                                        })
                                    {
                                        stats.replay_complete.fetch_add(1, Ordering::Relaxed);
                                        stats.replay_max_ms.fetch_max(
                                            opened.elapsed().as_millis() as u64,
                                            Ordering::Relaxed,
                                        );
                                        replay_reported = true;
                                    }
                                }
                                Err(_) => {
                                    stats.invalid.fetch_add(1, Ordering::Relaxed);
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Ping(_))) => {
                        if !matches!(
                            tokio::time::timeout(Duration::from_secs(5), socket.flush()).await,
                            Ok(Ok(()))
                        ) {
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {}
                    _ => break,
                }
            }
            if !intentional_close && tokio::time::Instant::now() < end {
                stats.unexpected_disconnects.fetch_add(1, Ordering::Relaxed);
            }
            if opened.elapsed() >= Duration::from_secs(30) {
                attempt = 0;
            }
        } else {
            stats.failed_attempts.fetch_add(1, Ordering::Relaxed);
        }
        let delay = jitter(&mut seed, attempt);
        attempt = attempt.saturating_add(1);
        tokio::time::sleep_until((tokio::time::Instant::now() + delay).min(end)).await;
    }
    if let Some(expect) = expectations.read().await.clone() {
        let matches = if timer {
            expect
                .timer
                .as_ref()
                .is_none_or(|value| latest_timer.as_ref() == Some(value))
        } else {
            expect
                .runs
                .iter()
                .all(|run| latest.get(&run.id) == Some(&fingerprint(run)))
                && latest.len() == expect.runs.len()
        };
        stats.validated.fetch_add(1, Ordering::Relaxed);
        if !matches {
            stats.mismatch.fetch_add(1, Ordering::Relaxed);
        }
    }
    Ok(())
}
async fn run(args: Args) -> Result<(), String> {
    if args.ramp_seconds.saturating_add(args.hold_seconds) > 86400 {
        return Err("workload duration must not exceed one day".into());
    }
    if args.viewers == 0 || args.hold_seconds == 0 || args.seed == 0 || args.max_run_ids == 0 {
        return Err("viewers, hold-seconds, max-run-ids and seed must be positive".into());
    }
    if args.target.host_str().is_none()
        || !matches!(args.target.scheme(), "http" | "https")
        || !args.target.username().is_empty()
        || args.target.password().is_some()
        || args.target.query().is_some()
        || args.target.fragment().is_some()
    {
        return Err("target must be an HTTP(S) base URL without credentials/query/fragment".into());
    }
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let connector = if let Some(ca) = &args.ca {
        let mut roots = rustls::RootCertStore::empty();
        let bytes = tokio::fs::read(ca).await.map_err(|e| e.to_string())?;
        for cert in rustls_pemfile::certs(&mut &bytes[..]) {
            roots
                .add(cert.map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
        }
        Some(Connector::Rustls(Arc::new(
            rustls::ClientConfig::builder()
                .with_root_certificates(roots)
                .with_no_client_auth(),
        )))
    } else {
        None
    };
    let expectations = Arc::new(tokio::sync::RwLock::new(
        expected(&args.expect_file).await?.map(Arc::new),
    ));
    let watch_state = expectations.clone();
    let watch_path = args.expect_file.clone();
    let watcher = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            if let Ok(Some(value)) = expected(&watch_path).await {
                *watch_state.write().await = Some(Arc::new(value));
            }
        }
    });
    let args = Arc::new(args);
    let stats = Arc::new(Stats::default());
    let start = tokio::time::Instant::now();
    let samples = Arc::new(std::sync::Mutex::new(Vec::new()));
    let sample_values = samples.clone();
    let sample_stats = stats.clone();
    let sampler = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            sample_values.lock().unwrap().push(Sample {
                elapsed_seconds: start.elapsed().as_secs_f64(),
                active_sockets: sample_stats.active.load(Ordering::Relaxed),
                opened_sockets: sample_stats.opened.load(Ordering::Relaxed),
                failed_attempts: sample_stats.failed_attempts.load(Ordering::Relaxed),
            });
        }
    });
    let (trace, writer) = if let Some(path) = &args.trace {
        use tokio::io::AsyncWriteExt;
        let file = tokio::fs::File::create(path)
            .await
            .map_err(|e| e.to_string())?;
        let (sender, mut receiver) = tokio::sync::mpsc::channel::<String>(4096);
        let writer = tokio::spawn(async move {
            let mut file = tokio::io::BufWriter::new(file);
            while let Some(line) = receiver.recv().await {
                file.write_all(line.as_bytes()).await?;
                file.write_all(b"\n").await?;
            }
            file.flush().await
        });
        (Some(sender), Some(writer))
    } else {
        (None, None)
    };
    let mut tasks = tokio::task::JoinSet::new();
    for id in 0..args.viewers {
        for timer in [false, true] {
            tasks.spawn(stream(
                args.clone(),
                stats.clone(),
                connector.clone(),
                (id, timer),
                start,
                expectations.clone(),
                trace.clone(),
            ));
        }
    }
    let mut errors = 0;
    while let Some(result) = tasks.join_next().await {
        if !matches!(result, Ok(Ok(()))) {
            errors += 1;
        }
    }
    watcher.abort();
    sampler.abort();
    drop(trace);
    if let Some(writer) = writer {
        writer
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;
    }
    expected(&args.expect_file).await?;
    let load = |value: &AtomicU64| value.load(Ordering::Relaxed);
    let report = Report {
        version: env!("CARGO_PKG_VERSION"),
        target: args.target.to_string(),
        event: args.event.clone(),
        contest: args.contest.clone(),
        viewers: args.viewers,
        ramp_seconds: args.ramp_seconds,
        hold_seconds: args.hold_seconds,
        reconnect: args.reconnect.clone(),
        seed: args.seed,
        elapsed_seconds: start.elapsed().as_secs_f64(),
        peak_sockets: load(&stats.peak),
        opened_sockets: load(&stats.opened),
        failed_attempts: load(&stats.failed_attempts),
        messages: load(&stats.messages),
        invalid_messages: load(&stats.invalid),
        final_mismatches: load(&stats.mismatch),
        validated_streams: load(&stats.validated),
        completed_initial_replays: load(&stats.replay_complete),
        max_initial_replay_ms: load(&stats.replay_max_ms),
        expected_state_checked: args.expect_file.is_some(),
        unexpected_disconnects: load(&stats.unexpected_disconnects),
        trace_dropped: load(&stats.trace_dropped),
        samples: std::mem::take(&mut *samples.lock().unwrap()),
        capacity_claim: "This workload report alone does not establish production capacity; correlate with server and host metrics.",
    };
    tokio::fs::write(&args.output, serde_json::to_vec_pretty(&report).unwrap())
        .await
        .map_err(|e| e.to_string())?;
    if errors > 0
        || report.invalid_messages > 0
        || report.final_mismatches > 0
        || report.trace_dropped > 0
        || report.unexpected_disconnects > 0
        || report.peak_sockets < (args.viewers as u64) * 2
    {
        return Err(format!(
            "workload failed; inspect {} (task errors: {errors})",
            args.output.display()
        ));
    }
    Ok(())
}
#[tokio::main]
async fn main() -> std::process::ExitCode {
    match run(Args::parse()).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paths_encode_identifiers_and_jitter_is_bounded() {
        let args = Args::parse_from([
            "viewer-load",
            "--target",
            "https://example.com",
            "--event",
            "a/b",
            "--contest",
            "c",
        ]);
        assert!(path(&args, false).contains("a%2Fb/contests/c/runs_ws"));
        let mut seed = 1;
        for attempt in 0..10 {
            let ms = jitter(&mut seed, attempt).as_millis();
            let cap = (1000u64 << attempt.min(5)).min(30000) as u128;
            assert!(ms >= cap / 2 && ms <= cap);
        }
    }
}
