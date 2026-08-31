use sentry::ClientInitGuard;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Initializes Sentry (DSN from the `SENTRY_DSN` env var) and the tracing
/// subscriber with the fmt and Sentry layers, so ERROR-level logs reach
/// Sentry with their span context; Sentry also installs its panic hook.
pub fn setup() -> ClientInitGuard {
    let guard = sentry::init(
        sentry::ClientOptions::default().release(sentry::release_name!().unwrap_or_default()),
    );

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(sentry_tracing::layer())
        .init();

    guard
}
