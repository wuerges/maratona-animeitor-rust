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
        .with(tracing_subscriber::fmt::layer().with_filter(filter))
        .with(sentry_tracing::layer())
}

/// Capture a fatal returned error while the initialization guard is still alive.
pub fn report_failure(error: &(dyn std::error::Error + 'static)) {
    tracing::error!(error, "application failed");
}

#[cfg(test)]
mod tests {
    use super::*;

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
