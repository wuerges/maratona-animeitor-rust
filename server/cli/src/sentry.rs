use sentry::ClientInitGuard;

pub fn setup() -> ClientInitGuard {
    sentry::init(
        sentry::ClientOptions::default().release(sentry::release_name!().unwrap_or_default()),
    )
}
