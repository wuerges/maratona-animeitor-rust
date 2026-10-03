use sentry::ClientInitGuard;
use tracing_subscriber::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Initializes Sentry (DSN from the `SENTRY_DSN` env var) and the tracing
/// subscriber with the fmt and Sentry layers, so ERROR-level logs reach
/// Sentry with their span context; Sentry also installs its panic hook.
pub fn setup() -> ClientInitGuard {
    let guard = sentry::init(options());

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    subscriber(filter).init();
    if !guard.is_enabled() {
        eprintln!("Sentry reporting is disabled: SENTRY_DSN is missing or invalid");
    }

    guard
}

fn options() -> sentry::ClientOptions {
    sentry::ClientOptions::default()
        .release(sentry::release_name!().unwrap_or_default())
        .attach_stacktrace(true)
}

fn subscriber(filter: tracing_subscriber::EnvFilter) -> impl tracing::Subscriber + Send + Sync {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().json().with_filter(filter))
        .with(sentry_tracing::layer())
}

/// Capture a fatal returned error while the initialization guard is still alive.
pub fn report_failure(error: &(dyn std::error::Error + 'static)) {
    let mut cause = Some(error);
    while let Some(error) = cause {
        if error.is::<crate::configuration::MissingWebcast>() {
            return;
        }
        cause = error.source();
    }
    let mut event = sentry::event_from_error(error);
    if let Some(exception) = event.exception.last_mut() {
        exception.stacktrace = sentry::integrations::backtrace::current_stacktrace();
    }
    sentry::capture_event(event);
}

/// Report unexpected failures to Sentry and render a readable diagnostic on stderr, independently
/// of the log filter. Do not send the terminal rendering through tracing.
pub fn report_cli_failure(error: &color_eyre::eyre::Report) {
    report_failure(error.as_ref());
    eprintln!("Error: {error}");
    for cause in error.chain().skip(1) {
        eprintln!("\nCaused by: {cause}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_webcast_is_not_sent_to_sentry() {
        let events = sentry::test::with_captured_events_options(
            || {
                let error = color_eyre::eyre::Report::new(crate::configuration::MissingWebcast {
                    message: "missing webcast mapping".into(),
                });
                report_failure(error.as_ref());
                report_failure(error.wrap_err("loading feeder configuration").as_ref());
            },
            sentry::apply_defaults(options()),
        );
        assert!(events.is_empty());
    }

    #[test]
    fn json_logs_escape_line_breaks() {
        #[derive(Clone, Default)]
        struct Buffer(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
        impl std::io::Write for Buffer {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let buffer = Buffer::default();
        let writer = buffer.clone();
        let subscriber = tracing_subscriber::fmt()
            .json()
            .with_writer(move || writer.clone())
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            let error = std::io::Error::other("first line\nsecond line\r\nthird line");
            tracing::error!(error = &error as &dyn std::error::Error, "request failed");
        });
        let output = String::from_utf8(buffer.0.lock().unwrap().clone()).unwrap();
        assert_eq!(output.lines().count(), 1);
        let entry: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(
            entry["fields"]["error"],
            "first line\nsecond line\r\nthird line"
        );
    }

    #[test]
    fn captures_errors_and_panics_when_console_is_disabled() {
        let _subscriber = subscriber(tracing_subscriber::EnvFilter::new("off")).set_default();
        let events = sentry::test::with_captured_events_options(
            || {
                service::database::report_error(&service::database::DatabaseError::NotFound);
                service::database::report_error(&service::database::DatabaseError::AlreadyExists);
                report_failure(&std::io::Error::other("startup database failure"));
                let _ = std::panic::catch_unwind(|| panic!("test panic capture"));
            },
            sentry::apply_defaults(options()),
        );
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].level, sentry::Level::Error);
        assert!(
            events[0]
                .exception
                .iter()
                .any(|e| e.value.as_deref() == Some("startup database failure"))
        );
        assert!(events[0].exception.iter().any(|e| e.stacktrace.is_some()));
        assert_eq!(events[1].level, sentry::Level::Fatal);
        assert_eq!(
            events[1].exception[0].value.as_deref(),
            Some("test panic capture")
        );
        assert!(events[1].exception[0].stacktrace.is_some());
    }
}
