use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(title = "Animeitor public API", version = "2.1.0"),
    paths(
        public_events, public_contests, public_contest, public_config,
        public_runs_ws, public_runs_secret, public_timer, public_remote_control
    ),
    components(schemas(
        data::event::TeamInfo, data::event::Answer, data::event::Run,
        data::event::PublicTimer,
        data::event::PublicContestState, data::event::PublicSiteView,
        data::event::PublicConfig
    )),
    modifiers(&PublicSecurityModifier)
)]
pub struct PublicApiDoc;

#[derive(OpenApi)]
#[openapi(
    info(title = "Animeitor internal API", version = "2.1.0"),
    paths(
        internal_events, internal_event, internal_contests, internal_contest,
        internal_list_sites,
        internal_site,
        internal_time, internal_runs, internal_event_salt,
        internal_contest_salt, internal_site_salt, internal_metrics
    ),
    components(schemas(
        data::event::TeamInfo, data::event::EventState,
        data::event::ContestConfig, data::event::SiteConfig,
        data::event::Answer, data::event::Run
    )),
    security(("basicAuth" = [])),
    modifiers(&SecurityModifier)
)]
pub struct InternalApiDoc;

struct SecurityModifier;
struct PublicSecurityModifier;
impl utoipa::Modify for PublicSecurityModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
        }
    }
}

impl utoipa::Modify for SecurityModifier {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let security = utoipa::openapi::security::SecurityScheme::Http(
            utoipa::openapi::security::Http::new(utoipa::openapi::security::HttpAuthScheme::Basic),
        );
        openapi.components = openapi.components.take().map(|mut components| {
            components.add_security_scheme("basicAuth", security);
            components.add_security_scheme(
                "bearerAuth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::Http::new(
                        utoipa::openapi::security::HttpAuthScheme::Bearer,
                    ),
                ),
            );
            components
        });
    }
}

fn response() -> (u16, &'static str) {
    (200, "Successful response")
}

#[utoipa::path(get, path = "/api/events", responses((status = 200, description = "Events")))]
pub async fn public_events() -> (u16, &'static str) {
    response()
}
#[utoipa::path(get, path = "/api/events/{event_name}/contests", params(("event_name" = String, Path)), responses((status = 200, description = "Contests")))]
pub async fn public_contests() -> (u16, &'static str) {
    response()
}
#[utoipa::path(get, path = "/api/events/{event_name}/contests/{contest_name}/contest", params(("event_name" = String, Path), ("contest_name" = String, Path)), responses((status = 200, description = "Contest state")))]
pub async fn public_contest() -> (u16, &'static str) {
    response()
}
#[utoipa::path(get, path = "/api/events/{event_name}/contests/{contest_name}/config", params(("event_name" = String, Path), ("contest_name" = String, Path)), responses((status = 200, description = "Contest config")))]
pub async fn public_config() -> (u16, &'static str) {
    response()
}
#[utoipa::path(get, path = "/api/events/{event_name}/contests/{contest_name}/runs_ws", params(("event_name" = String, Path), ("contest_name" = String, Path)), responses((status = 101, description = "WebSocket")))]
pub async fn public_runs_ws() -> (u16, &'static str) {
    response()
}
#[utoipa::path(get, path = "/api/events/{event_name}/contests/{contest_name}/runs_secret", params(("event_name" = String, Path), ("contest_name" = String, Path)), security(("bearerAuth" = [])), responses((status = 200, description = "Secret runs")))]
pub async fn public_runs_secret() -> (u16, &'static str) {
    response()
}
#[utoipa::path(get, path = "/api/events/{event_name}/timer", params(("event_name" = String, Path)), responses((status = 101, description = "WebSocket")))]
pub async fn public_timer() -> (u16, &'static str) {
    response()
}
#[utoipa::path(get, path = "/api/events/{event_name}/contests/{contest_name}/remote_control/{key}", params(("event_name" = String, Path), ("contest_name" = String, Path), ("key" = String, Path)), responses((status = 101, description = "WebSocket")))]
pub async fn public_remote_control() -> (u16, &'static str) {
    response()
}

#[utoipa::path(get, path = "/internal/events", responses((status = 200, description = "Events")))]
pub async fn internal_events() -> (u16, &'static str) {
    response()
}
#[utoipa::path(get, post, put, delete, path = "/internal/events/{event_name}", params(("event_name" = String, Path)), responses((status = 200, description = "Event")))]
pub async fn internal_event() -> (u16, &'static str) {
    response()
}
#[utoipa::path(get, path = "/internal/events/{event_name}/contests", params(("event_name" = String, Path)), responses((status = 200, description = "Contests")))]
pub async fn internal_contests() -> (u16, &'static str) {
    response()
}
#[utoipa::path(get, post, put, delete, path = "/internal/contests/{event_name}/{contest_name}", params(("event_name" = String, Path), ("contest_name" = String, Path)), responses((status = 200, description = "Contest")))]
pub async fn internal_contest() -> (u16, &'static str) {
    response()
}
#[utoipa::path(get, path = "/internal/events/{event_name}/contests/{contest_name}/sites", params(("event_name" = String, Path), ("contest_name" = String, Path)), responses((status = 200, description = "Sites")))]
pub async fn internal_list_sites() -> (u16, &'static str) {
    response()
}
#[utoipa::path(post, put, delete, path = "/internal/sites/{event_name}/{contest_name}/{site_name}", params(("event_name" = String, Path), ("contest_name" = String, Path), ("site_name" = String, Path)), responses((status = 200, description = "Site")))]
pub async fn internal_site() -> (u16, &'static str) {
    response()
}
#[utoipa::path(patch, path = "/internal/events/{event_name}/time", params(("event_name" = String, Path)), responses((status = 200, description = "Time")))]
pub async fn internal_time() -> (u16, &'static str) {
    response()
}
#[utoipa::path(post, delete, path = "/internal/events/{event_name}/runs", params(("event_name" = String, Path)), responses((status = 200, description = "Runs")))]
pub async fn internal_runs() -> (u16, &'static str) {
    response()
}
#[utoipa::path(post, path = "/internal/events/{event_name}/salt", params(("event_name" = String, Path)), responses((status = 200, description = "Salt")))]
pub async fn internal_event_salt() -> (u16, &'static str) {
    response()
}
#[utoipa::path(post, path = "/internal/contests/{event_name}/{contest_name}/salt", params(("event_name" = String, Path), ("contest_name" = String, Path)), responses((status = 200, description = "Salt")))]
pub async fn internal_contest_salt() -> (u16, &'static str) {
    response()
}
#[utoipa::path(post, path = "/internal/sites/{event_name}/{contest_name}/{site_name}/salt", params(("event_name" = String, Path), ("contest_name" = String, Path), ("site_name" = String, Path)), responses((status = 200, description = "Salt")))]
pub async fn internal_site_salt() -> (u16, &'static str) {
    response()
}
#[utoipa::path(get, path = "/internal/metrics", responses((status = 200, description = "Prometheus metrics")))]
pub async fn internal_metrics() -> (u16, &'static str) {
    response()
}

pub fn swagger_html(spec: &str) -> String {
    format!(
        r##"<!doctype html><html><head><title>Animeitor API</title><link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist/swagger-ui.css"></head><body><div id="swagger-ui"></div><script src="https://unpkg.com/swagger-ui-dist/swagger-ui-bundle.js"></script><script>SwaggerUIBundle({{url:"{spec}",dom_id:"#swagger-ui"}})</script></body></html>"##
    )
}
