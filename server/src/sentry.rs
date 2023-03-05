use sentry::ClientInitGuard;

pub fn setup() -> ClientInitGuard {
    sentry::init(sentry::ClientOptions {
        release: sentry::release_name!(),
        ..Default::default()
    })
}
