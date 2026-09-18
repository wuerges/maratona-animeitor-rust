//! OpenAPI contracts. Documentation-only handlers below mirror the real routers.
use crate::internal::{RunsBody, SaltBody, TimeBody};
use data::event::*;
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

/// Successful JSON response. Requests do not use this envelope.
#[derive(Serialize, ToSchema)]
struct Success<T> {
    /// Operation result; required on success.
    data: T,
    /// Nonfatal issues, omitted when absent (currently unknown_team on run ingestion).
    #[serde(skip_serializing_if = "Option::is_none")]
    warnings: Option<Vec<ErrorEntry>>,
}
/// Error response; data and warnings are absent.
#[derive(Serialize, ToSchema)]
struct Failure {
    errors: Vec<ErrorEntry>,
}
#[derive(Serialize, ToSchema)]
struct RunCounts {
    /// Number of new submission IDs.
    added: usize,
    /// Number of changed submissions; identical resends do not count.
    updated: usize,
}
#[derive(Serialize, ToSchema)]
struct SaltData {
    /// Effective salt after the operation, including generated values.
    salt: String,
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Animeitor public API", version = "2.1.0"),
    paths(list_public_events,
        list_public_contests,
        public_contest_state,
        public_contest_config,
        public_secret_runs,
        public_runs_stream,
        public_timer_stream,
        public_remote_control),
    components(schemas(TeamInfo, Answer, Run, ErrorEntry, PublicTimer, PublicContestState, PublicConfig, PublicSiteView, RunsData, data::remote_control::ControlMessage)),
    modifiers(&PublicMetadata)
)]
pub struct PublicApiDoc;

#[derive(OpenApi)]
#[openapi(
    info(title = "Animeitor internal API", version = "2.1.0"),
    paths(list_internal_events,
        get_internal_event,
        post_internal_event,
        put_internal_event,
        delete_internal_event,
        list_internal_contests,
        list_internal_sites,
        post_internal_contest,
        put_internal_contest,
        delete_internal_contest,
        post_internal_site,
        put_internal_site,
        delete_internal_site,
        update_event_time,
        ingest_runs,
        clear_runs,
        rotate_event_salt,
        rotate_contest_salt,
        rotate_site_salt,
        list_revelation_urls,
        internal_metrics),
    components(schemas(TeamInfo, Answer, Run, ErrorEntry, EventState, ContestConfig, SiteConfig, RevelationUrl, TimeBody, RunsBody, SaltBody)),
    modifiers(&InternalMetadata),
    security(("basicAuth" = []))
)]
pub struct InternalApiDoc;

struct InternalMetadata;
struct PublicMetadata;
impl utoipa::Modify for InternalMetadata {
    fn modify(&self, doc: &mut utoipa::openapi::OpenApi) {
        doc.info.description = Some(include_str!("../../../doc/internal-api-setup.md").into());
        // An absent salt body is valid, but a JSON null body is not a SaltBody.
        // Mark the request optional without making its schema nullable.
        for (path, item) in &mut doc.paths.paths {
            if path.ends_with("/salt") {
                if let Some(body) = item.post.as_mut().and_then(|op| op.request_body.as_mut()) {
                    body.required = Some(utoipa::openapi::Required::False);
                }
            }
        }

        doc.components.as_mut().unwrap().add_security_scheme(
            "basicAuth",
            utoipa::openapi::security::SecurityScheme::Http(utoipa::openapi::security::Http::new(
                utoipa::openapi::security::HttpAuthScheme::Basic,
            )),
        );
    }
}
impl utoipa::Modify for PublicMetadata {
    fn modify(&self, doc: &mut utoipa::openapi::OpenApi) {
        doc.info.description = Some(include_str!("../../../doc/public-api-overview.md").into());
        doc.components.as_mut().unwrap().add_security_scheme(
            "bearerAuth",
            utoipa::openapi::security::SecurityScheme::Http(utoipa::openapi::security::Http::new(
                utoipa::openapi::security::HttpAuthScheme::Bearer,
            )),
        );
    }
}

/// List configured events
///
/// Returns event identifiers in creation order, including events before start. State is in memory and is lost when the server restarts.
#[utoipa::path(
    get, path = "/internal/events", operation_id = "list_internal_events", tag = "Events",
    responses(
        (status = 200, description = "Successful result in data", body = Success<Vec<String>>, example = json!({"data": ["regional-2026"]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn list_internal_events() {}

/// Read complete event state
///
/// Available before start. Includes salts and the complete roster, but not contests, sites, or runs. Use the nested list operations to inspect contests and sites.
#[utoipa::path(
    get, path = "/internal/events/{event_name}", operation_id = "get_internal_event", tag = "Events",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment.")),
    responses(
        (status = 200, description = "Successful result in data", body = Success<EventState>, example = json!({"data": {"name": "regional-2026", "problems": ["A", "B"], "teams": [{"login": "teambr001", "escola": "Example University", "nome": "Example Team"}], "score_freeze_time_seconds": 14400, "penalty_seconds": 1200, "time_seconds": -60, "salt": null}})),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn get_internal_event() {}

/// Create event state
///
/// Create the event before creating contests or sending runs. The body name must equal event_name. There is no automatic default contest.
#[utoipa::path(
    post, path = "/internal/events/{event_name}", operation_id = "post_internal_event", tag = "Events",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment.")),
    request_body(content = EventState, content_type = "application/json", example = json!({"name": "regional-2026", "problems": ["A", "B"], "teams": [{"login": "teambr001", "escola": "Example University", "nome": "Example Team"}], "score_freeze_time_seconds": 14400, "penalty_seconds": 1200, "time_seconds": -60})),
    responses(
        (status = 201, description = "Successful result in data", body = Success<EventState>, example = json!({"data": {"name": "regional-2026", "problems": ["A", "B"], "teams": [{"login": "teambr001", "escola": "Example University", "nome": "Example Team"}], "score_freeze_time_seconds": 14400, "penalty_seconds": 1200, "time_seconds": -60, "salt": null}})),
        (status = 400, description = "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex.", body = Failure, example = json!({"errors": [{"code": "invalid_value", "message": "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex."}]})),
        (status = 409, description = "Resource already exists; inspect it before choosing to replace it.", body = Failure, example = json!({"errors": [{"code": "conflict", "message": "Resource already exists; inspect it before choosing to replace it."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn post_internal_event() {}

/// Replace event state
///
/// Replaces event fields, preserving existing contests, sites, and runs. The body name must equal event_name. Omitted optional fields reset to defaults: time_seconds to 0 and salt to null. Read the existing state first to preserve values. This is not a merge or an upsert.
#[utoipa::path(
    put, path = "/internal/events/{event_name}", operation_id = "put_internal_event", tag = "Events",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment.")),
    request_body(content = EventState, content_type = "application/json", example = json!({"name": "regional-2026", "problems": ["A", "B"], "teams": [{"login": "teambr001", "escola": "Example University", "nome": "Example Team"}], "score_freeze_time_seconds": 14400, "penalty_seconds": 1200, "time_seconds": -60})),
    responses(
        (status = 200, description = "Successful result in data", body = Success<EventState>, example = json!({"data": {"name": "regional-2026", "problems": ["A", "B"], "teams": [{"login": "teambr001", "escola": "Example University", "nome": "Example Team"}], "score_freeze_time_seconds": 14400, "penalty_seconds": 1200, "time_seconds": -60, "salt": null}})),
        (status = 400, description = "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex.", body = Failure, example = json!({"errors": [{"code": "invalid_value", "message": "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex."}]})),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn put_internal_event() {}

/// Delete event and all children
///
/// Deletes the event, all its contests and sites, and stored runs. This operation is destructive; no response body.
#[utoipa::path(
    delete, path = "/internal/events/{event_name}", operation_id = "delete_internal_event", tag = "Events",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment.")),
    responses(
        (status = 204, description = "Deleted; no response body."),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn delete_internal_event() {}

/// List complete contest configurations
///
/// Returns configuration objects, including salts, in unspecified order. Available before start. There is no individual contest GET route.
#[utoipa::path(
    get, path = "/internal/events/{event_name}/contests", operation_id = "list_internal_contests", tag = "Contests",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment.")),
    responses(
        (status = 200, description = "Successful result in data", body = Success<Vec<ContestConfig>>, example = json!({"data": [{"name": "brasil", "codes": ["^teambr"], "ouro": 1, "prata": 2, "bronze": 3, "photo_url_format": "https://media.example.com/photos/{team_login}.webp", "sound_url_format": "https://media.example.com/sounds/{team_login}.mp3", "salt": null, "style": null}]})),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn list_internal_contests() {}

/// List complete site configurations
///
/// Returns configuration objects, including salts, in unspecified order. Available before start. There is no individual site GET route.
#[utoipa::path(
    get, path = "/internal/events/{event_name}/contests/{contest_name}/sites", operation_id = "list_internal_sites", tag = "Sites",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment."), ("contest_name" = String, Path, description = "Nonempty contest identifier within the event. URL-encode as a path segment.")),
    responses(
        (status = 200, description = "Successful result in data", body = Success<Vec<SiteConfig>>, example = json!({"data": [{"name": "fiemg", "codes": ["^teambr001$"], "salt": null}]})),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn list_internal_sites() {}

/// Create contest
///
/// Requires the parent event to exist. Creates a new contest. codes uses Rust regex syntax and OR matching, not literal prefixes; use anchors for exact matching. Empty codes matches no teams. The body name must be nonempty and equal the path name.
#[utoipa::path(
    post, path = "/internal/contests/{event_name}/{contest_name}", operation_id = "post_internal_contest", tag = "Contests",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment."), ("contest_name" = String, Path, description = "Nonempty contest identifier within the event. URL-encode as a path segment.")),
    request_body(content = ContestConfig, content_type = "application/json", example = json!({"name": "brasil", "codes": ["^teambr"], "ouro": 1, "prata": 2, "bronze": 3, "photo_url_format": "https://media.example.com/photos/{team_login}.webp", "sound_url_format": "https://media.example.com/sounds/{team_login}.mp3"})),
    responses(
        (status = 201, description = "Successful result in data", body = Success<ContestConfig>, example = json!({"data": {"name": "brasil", "codes": ["^teambr"], "ouro": 1, "prata": 2, "bronze": 3, "photo_url_format": "https://media.example.com/photos/{team_login}.webp", "sound_url_format": "https://media.example.com/sounds/{team_login}.mp3", "salt": null, "style": null}})),
        (status = 400, description = "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex.", body = Failure, example = json!({"errors": [{"code": "invalid_value", "message": "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex."}]})),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]})),
        (status = 409, description = "Resource already exists; inspect it before choosing to replace it.", body = Failure, example = json!({"errors": [{"code": "conflict", "message": "Resource already exists; inspect it before choosing to replace it."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn post_internal_contest() {}

/// Replace contest
///
/// Requires the parent event to exist. Replaces all configuration fields; omitted optional fields reset to their defaults. Not a merge or an upsert. Existing sites are preserved. codes uses Rust regex syntax and OR matching, not literal prefixes; use anchors for exact matching. Empty codes matches no teams. The body name must be nonempty and equal the path name.
#[utoipa::path(
    put, path = "/internal/contests/{event_name}/{contest_name}", operation_id = "put_internal_contest", tag = "Contests",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment."), ("contest_name" = String, Path, description = "Nonempty contest identifier within the event. URL-encode as a path segment.")),
    request_body(content = ContestConfig, content_type = "application/json", example = json!({"name": "brasil", "codes": ["^teambr"], "ouro": 1, "prata": 2, "bronze": 3, "photo_url_format": "https://media.example.com/photos/{team_login}.webp", "sound_url_format": "https://media.example.com/sounds/{team_login}.mp3"})),
    responses(
        (status = 200, description = "Successful result in data", body = Success<ContestConfig>, example = json!({"data": {"name": "brasil", "codes": ["^teambr"], "ouro": 1, "prata": 2, "bronze": 3, "photo_url_format": "https://media.example.com/photos/{team_login}.webp", "sound_url_format": "https://media.example.com/sounds/{team_login}.mp3", "salt": null, "style": null}})),
        (status = 400, description = "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex.", body = Failure, example = json!({"errors": [{"code": "invalid_value", "message": "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex."}]})),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn put_internal_contest() {}

/// Delete contest
///
/// Deletes the contest and all its sites; event runs are retained. No response body.
#[utoipa::path(
    delete, path = "/internal/contests/{event_name}/{contest_name}", operation_id = "delete_internal_contest", tag = "Contests",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment."), ("contest_name" = String, Path, description = "Nonempty contest identifier within the event. URL-encode as a path segment.")),
    responses(
        (status = 204, description = "Deleted; no response body."),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn delete_internal_contest() {}

/// Create site
///
/// Requires the parent event and contest to exist. Creates a new site. codes uses Rust regex syntax and OR matching, not literal prefixes; use anchors for exact matching. Empty codes matches no teams. Use the path name in the body; empty body names are normalized in storage (the write response echoes the submitted config). Site filters should be a subset of contest teams; this is not enforced for secret runs.
#[utoipa::path(
    post, path = "/internal/sites/{event_name}/{contest_name}/{site_name}", operation_id = "post_internal_site", tag = "Sites",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment."), ("contest_name" = String, Path, description = "Nonempty contest identifier within the event. URL-encode as a path segment."), ("site_name" = String, Path, description = "Site identifier within the contest. URL-encode as a path segment.")),
    request_body(content = SiteConfig, content_type = "application/json", example = json!({"name": "fiemg", "codes": ["^teambr001$"]})),
    responses(
        (status = 201, description = "Successful result in data", body = Success<SiteConfig>, example = json!({"data": {"name": "fiemg", "codes": ["^teambr001$"], "salt": null}})),
        (status = 400, description = "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex.", body = Failure, example = json!({"errors": [{"code": "invalid_value", "message": "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex."}]})),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]})),
        (status = 409, description = "Resource already exists; inspect it before choosing to replace it.", body = Failure, example = json!({"errors": [{"code": "conflict", "message": "Resource already exists; inspect it before choosing to replace it."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn post_internal_site() {}

/// Replace site
///
/// Requires the parent event and contest to exist. Replaces all configuration fields; omitted optional fields reset to their defaults. Not a merge or an upsert.  codes uses Rust regex syntax and OR matching, not literal prefixes; use anchors for exact matching. Empty codes matches no teams. Use the path name in the body; empty body names are normalized in storage (the write response echoes the submitted config). Site filters should be a subset of contest teams; this is not enforced for secret runs.
#[utoipa::path(
    put, path = "/internal/sites/{event_name}/{contest_name}/{site_name}", operation_id = "put_internal_site", tag = "Sites",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment."), ("contest_name" = String, Path, description = "Nonempty contest identifier within the event. URL-encode as a path segment."), ("site_name" = String, Path, description = "Site identifier within the contest. URL-encode as a path segment.")),
    request_body(content = SiteConfig, content_type = "application/json", example = json!({"name": "fiemg", "codes": ["^teambr001$"]})),
    responses(
        (status = 200, description = "Successful result in data", body = Success<SiteConfig>, example = json!({"data": {"name": "fiemg", "codes": ["^teambr001$"], "salt": null}})),
        (status = 400, description = "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex.", body = Failure, example = json!({"errors": [{"code": "invalid_value", "message": "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex."}]})),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn put_internal_site() {}

/// Delete site
///
/// Deletes the site; event runs are retained. No response body.
#[utoipa::path(
    delete, path = "/internal/sites/{event_name}/{contest_name}/{site_name}", operation_id = "delete_internal_site", tag = "Sites",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment."), ("contest_name" = String, Path, description = "Nonempty contest identifier within the event. URL-encode as a path segment."), ("site_name" = String, Path, description = "Site identifier within the contest. URL-encode as a path segment.")),
    responses(
        (status = 204, description = "Deleted; no response body."),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn delete_internal_site() {}

/// Set elapsed event time
///
/// Set the current elapsed seconds explicitly. Negative values keep public contest endpoints closed (countdown). At zero or above they become available. The server stores and publishes the supplied value; it does not tick automatically. Repeat updates from your controller or feeder.
#[utoipa::path(
    patch, path = "/internal/events/{event_name}/time", operation_id = "update_event_time", tag = "Timing and runs",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment.")),
    request_body(content = TimeBody, content_type = "application/json", example = json!({"time_seconds": 0})),
    responses(
        (status = 200, description = "Successful result in data", body = Success<TimeBody>, example = json!({"data": {"time_seconds": 0}})),
        (status = 400, description = "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex.", body = Failure, example = json!({"errors": [{"code": "invalid_value", "message": "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex."}]})),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn update_event_time() {}

/// Add or correct submissions
///
/// Requires an existing event. Runs from unknown teams are skipped with unknown_team warnings. Remaining runs are sorted by (time_seconds, id) and validated before applying; an unknown problem rejects the remaining batch. A new ID adds a run, a changed existing ID corrects it, and an identical resend is a no-op. Counts report additions and actual corrections. For multiple entries with the same ID in one batch, application follows the sorted order (equal sort keys preserve input order).
#[utoipa::path(
    post, path = "/internal/events/{event_name}/runs", operation_id = "ingest_runs", tag = "Timing and runs",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment.")),
    request_body(content = RunsBody, content_type = "application/json", example = json!({"runs": [{"id": 1, "team_login": "teambr001", "prob": "A", "time_seconds": 56, "answer": "Y"}]})),
    responses(
        (status = 200, description = "Successful result in data", body = Success<RunCounts>, example = json!({"data": {"added": 1, "updated": 0}})),
        (status = 400, description = "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex.", body = Failure, example = json!({"errors": [{"code": "invalid_value", "message": "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex."}]})),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn ingest_runs() {}

/// Remove all event submissions
///
/// Clears stored runs while preserving event, contest, and site configuration. Existing WebSocket replay history is retained and no reset message is sent, so reconnecting can replay previously cleared submissions. To reset stream history too, recreate the event and its configuration.
#[utoipa::path(
    delete, path = "/internal/events/{event_name}/runs", operation_id = "clear_runs", tag = "Timing and runs",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment.")),
    responses(
        (status = 204, description = "Deleted; no response body."),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn clear_runs() {}

/// Change event salt
///
/// Changes the salt for every site in the event. Existing revelation URLs for affected sites stop working; fetch the URL listing again. No body, {}, null salt, or an empty salt string generates a random value. A nonempty salt is used as supplied. Other fields are preserved. To clear a salt, PUT the complete configuration with salt omitted or null.
#[utoipa::path(
    post, path = "/internal/events/{event_name}/salt", operation_id = "rotate_event_salt", tag = "Revelation",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment.")),
    request_body(content = SaltBody, content_type = "application/json", example = json!({"salt": "new-example-salt"})),
    responses(
        (status = 200, description = "Successful result in data", body = Success<SaltData>, example = json!({"data": {"salt": "new-example-salt"}})),
        (status = 400, description = "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex.", body = Failure, example = json!({"errors": [{"code": "invalid_value", "message": "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex."}]})),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn rotate_event_salt() {}

/// Change contest salt
///
/// Changes the salt for every site in this contest. Existing revelation URLs for affected sites stop working; fetch the URL listing again. No body, {}, null salt, or an empty salt string generates a random value. A nonempty salt is used as supplied. Other fields are preserved. To clear a salt, PUT the complete configuration with salt omitted or null.
#[utoipa::path(
    post, path = "/internal/contests/{event_name}/{contest_name}/salt", operation_id = "rotate_contest_salt", tag = "Revelation",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment."), ("contest_name" = String, Path, description = "Nonempty contest identifier within the event. URL-encode as a path segment.")),
    request_body(content = SaltBody, content_type = "application/json", example = json!({"salt": "new-example-salt"})),
    responses(
        (status = 200, description = "Successful result in data", body = Success<SaltData>, example = json!({"data": {"salt": "new-example-salt"}})),
        (status = 400, description = "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex.", body = Failure, example = json!({"errors": [{"code": "invalid_value", "message": "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex."}]})),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn rotate_contest_salt() {}

/// Change site salt
///
/// Changes the salt for this site only. Existing revelation URLs for affected sites stop working; fetch the URL listing again. No body, {}, null salt, or an empty salt string generates a random value. A nonempty salt is used as supplied. Other fields are preserved. To clear a salt, PUT the complete configuration with salt omitted or null.
#[utoipa::path(
    post, path = "/internal/sites/{event_name}/{contest_name}/{site_name}/salt", operation_id = "rotate_site_salt", tag = "Revelation",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment."), ("contest_name" = String, Path, description = "Nonempty contest identifier within the event. URL-encode as a path segment."), ("site_name" = String, Path, description = "Site identifier within the contest. URL-encode as a path segment.")),
    request_body(content = SaltBody, content_type = "application/json", example = json!({"salt": "new-example-salt"})),
    responses(
        (status = 200, description = "Successful result in data", body = Success<SaltData>, example = json!({"data": {"salt": "new-example-salt"}})),
        (status = 400, description = "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex.", body = Failure, example = json!({"errors": [{"code": "invalid_value", "message": "Invalid JSON, missing required field, invalid value, or invalid regex. Codes: invalid_json, missing_field, invalid_value, invalid_regex."}]})),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn rotate_site_salt() {}

/// List full revelation URLs for every site
///
/// Returns one private frontend URL per site across all contests of this event, sorted by contest then site name. Available before start; an event without sites returns an empty list. URLs use the configured public_url origin and /animeitor/{event}/{contest}/, replacing any configured base path. The frontend must be deployed there. Parse the URL secret query parameter to obtain the Bearer token for the public runs_secret endpoint; sede selects the site. Treat the entire URL as a credential. Keys are derived from the current salts and private server secret in one event snapshot; retrieval does not rotate them. Fetch again after salt rotation. Responses use Cache-Control: no-store.
#[utoipa::path(
    get, path = "/internal/events/{event_name}/revelation_urls", operation_id = "list_revelation_urls", tag = "Revelation",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment.")),
    responses(
        (status = 200, description = "Successful result in data", headers(("Cache-Control" = String, description = "no-store")), body = Success<Vec<RevelationUrl>>, example = json!({"data": [{"contest": "brasil", "site": "fiemg", "url": "https://example.com/animeitor/regional-2026/brasil/?secret=EXAMPLE_KEY&sede=fiemg"}]})),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]})),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn list_revelation_urls() {}

/// Read process metrics
///
/// Process-wide Prometheus metrics, not scoped to an event. The response is text without a JSON envelope; encoding failures return an empty 500 response.
#[utoipa::path(
    get, path = "/internal/metrics", operation_id = "internal_metrics", tag = "Monitoring",
    responses(
        (status = 200, description = "Prometheus exposition text", body = String, content_type = "text/plain"),
        (status = 500, description = "Metrics encoding failed; no body"),
        (status = 401, description = "Missing or invalid configured username/token. WWW-Authenticate: Basic.", body = Failure, example = json!({"errors": [{"code": "unauthorized", "message": "Missing or invalid configured username/token. WWW-Authenticate: Basic."}]})),
        (status = 426, description = "Internal requests on the cleartext listener are rejected; use the configured HTTPS endpoint", body = String, content_type = "text/plain", example = "the internal API requires HTTPS")
    )
)]
pub async fn internal_metrics() {}

/// List event identifiers
///
/// Event identifiers in creation order, including pre-start events. No authentication.
#[utoipa::path(
    get, path = "/api/events", operation_id = "list_public_events", tag = "Public discovery",
    responses(
        (status = 200, description = "Successful result in data", body = Success<Vec<String>>, example = json!({"data": ["regional-2026"]}))
    )
)]
pub async fn list_public_events() {}

/// List contest identifiers
///
/// Alphabetically sorted contest names. Unlike the internal list this returns strings, not configurations. Unavailable before event start.
#[utoipa::path(
    get, path = "/api/events/{event_name}/contests", operation_id = "list_public_contests", tag = "Public discovery",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment.")),
    responses(
        (status = 200, description = "Successful result in data", body = Success<Vec<String>>, example = json!({"data": ["brasil"]})),
        (status = 403, description = "Before event start: not_started. For runs_secret also invalid_key when the key is missing or invalid.", body = Failure, example = json!({"errors": [{"code": "not_started", "message": "Before event start: not_started. For runs_secret also invalid_key when the key is missing or invalid."}]})),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]}))
    )
)]
pub async fn list_public_contests() {}

/// Read public contest state
///
/// Returns teams matching any contest codes regex, problem identifiers, and event timing. No salts or site keys. Before start returns 403 not_started, not a partial state.
#[utoipa::path(
    get, path = "/api/events/{event_name}/contests/{contest_name}/contest", operation_id = "public_contest_state", tag = "Public contest",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment."), ("contest_name" = String, Path, description = "Nonempty contest identifier within the event. URL-encode as a path segment.")),
    responses(
        (status = 200, description = "Successful result in data", body = Success<PublicContestState>, example = json!({"data": {"event": "regional-2026", "contest": "brasil", "problems": ["A", "B"], "teams": [{"login": "teambr001", "escola": "Example University", "nome": "Example Team"}], "score_freeze_time_seconds": 14400, "penalty_seconds": 1200, "time_seconds": 60}})),
        (status = 403, description = "Before event start: not_started. For runs_secret also invalid_key when the key is missing or invalid.", body = Failure, example = json!({"errors": [{"code": "not_started", "message": "Before event start: not_started. For runs_secret also invalid_key when the key is missing or invalid."}]})),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]}))
    )
)]
pub async fn public_contest_state() {}

/// Read public presentation configuration
///
/// Contest appearance, medal thresholds, media templates, and site choices. No salts or keys. The sites field is omitted when empty. Optional style and media values are null when unset. Unavailable before start.
#[utoipa::path(
    get, path = "/api/events/{event_name}/contests/{contest_name}/config", operation_id = "public_contest_config", tag = "Public contest",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment."), ("contest_name" = String, Path, description = "Nonempty contest identifier within the event. URL-encode as a path segment.")),
    responses(
        (status = 200, description = "Successful result in data", body = Success<PublicConfig>, example = json!({"data": {"name": "brasil", "codes": ["^teambr"], "ouro": 1, "prata": 2, "bronze": 3, "photo_url_format": "https://media.example.com/photos/{team_login}.webp", "sound_url_format": "https://media.example.com/sounds/{team_login}.mp3", "style": null, "sites": [{"name": "fiemg", "codes": ["^teambr001$"]}]}})),
        (status = 403, description = "Before event start: not_started. For runs_secret also invalid_key when the key is missing or invalid.", body = Failure, example = json!({"errors": [{"code": "not_started", "message": "Before event start: not_started. For runs_secret also invalid_key when the key is missing or invalid."}]})),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]}))
    )
)]
pub async fn public_contest_config() {}

/// Read actual submissions for a site
///
/// Send `Authorization: Bearer <site-key>`,  extracting the secret query parameter from an authenticated internal revelation URL listing. The key selects the site; no site path parameter is needed. Returns that site regex filter’s event runs, including unfrozen answers, ordered by (time_seconds, id). Ensure site filters select a subset of contest teams. No key works before start. Missing or invalid keys (including an unknown contest under a started event) return 403 invalid_key; a missing event returns 404. Salt rotation invalidates affected keys immediately.
#[utoipa::path(
    get, path = "/api/events/{event_name}/contests/{contest_name}/runs_secret", operation_id = "public_secret_runs", tag = "Public revelation",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment."), ("contest_name" = String, Path, description = "Nonempty contest identifier within the event. URL-encode as a path segment.")),
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Successful result in data", body = Success<RunsData>, example = json!({"data": {"runs": [{"id": 1, "team_login": "teambr001", "prob": "A", "time_seconds": 56, "answer": "Y"}]}})),
        (status = 403, description = "Before event start: not_started. For runs_secret also invalid_key when the key is missing or invalid.", body = Failure, example = json!({"errors": [{"code": "not_started", "message": "Before event start: not_started. For runs_secret also invalid_key when the key is missing or invalid."}]})),
        (status = 404, description = "Required event, contest, or site does not exist.", body = Failure, example = json!({"errors": [{"code": "not_found", "message": "Required event, contest, or site does not exist."}]}))
    )
)]
pub async fn public_secret_runs() {}

/// Stream public submissions
///
/// WebSocket upgrade (use ws:// or wss://). Replays submission messages from the event history, then sends updates. One bare Run JSON object per text frame; there is no envelope. Filtered by contest team regexes. Answers at or after score_freeze_time_seconds are replaced with ?. Repeated IDs replace previous values. Reconnect and rebuild state from replay after changing filters or freeze. Clearing stored runs does not clear stream replay history; a connection captures its filters and freeze boundary at handshake. Handshake failures for missing resources or pre-start events have no body.
#[utoipa::path(
    get, path = "/api/events/{event_name}/contests/{contest_name}/runs_ws", operation_id = "public_runs_stream", tag = "WebSockets",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment."), ("contest_name" = String, Path, description = "Nonempty contest identifier within the event. URL-encode as a path segment.")),
    responses(
        (status = 101, description = "WebSocket upgrade; subsequent text frames follow the message schema described above."),
        (status = 403, description = "Event has not started; no body."),
        (status = 404, description = "Resource not found; no body.")
    )
)]
pub async fn public_runs_stream() {}

/// Stream event time
///
/// WebSocket available before start. Sends a bare PublicTimer JSON object immediately, then changes with consecutive duplicates suppressed. Example: {"current_time_seconds":-60,"score_freeze_time_seconds":14400}. The stream reports caller-supplied time; the server does not advance it. Missing events return a bare 404 handshake response.
#[utoipa::path(
    get, path = "/api/events/{event_name}/timer", operation_id = "public_timer_stream", tag = "WebSockets",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment.")),
    responses(
        (status = 101, description = "WebSocket upgrade; subsequent text frames follow the message schema described above."),
        (status = 404, description = "Resource not found; no body.")
    )
)]
pub async fn public_timer_stream() {}

/// Relay frontend controls
///
/// WebSocket available before start. The key is an arbitrary shared relay-channel identifier, NOT a revelation key. Messages are relayed only to other connections with the same event, contest, and key. Text frames contain bare JSON matching ControlMessage: {"y":120}, {"query":"sede=fiemg"}, "Hidden", or {"Show":"teambr001"}. No outer WindowScroll, QueryString, or PhotoState wrapper. Missing resources return a bare 404 handshake response.
#[utoipa::path(
    get, path = "/api/events/{event_name}/contests/{contest_name}/remote_control/{key}", operation_id = "public_remote_control", tag = "WebSockets",
    params(("event_name" = String, Path, description = "Event identifier (not a display label). URL-encode as a path segment."), ("contest_name" = String, Path, description = "Nonempty contest identifier within the event. URL-encode as a path segment."), ("key" = String, Path, description = "Arbitrary shared remote-control channel identifier; not a revelation key.")),
    responses(
        (status = 101, description = "WebSocket upgrade; subsequent text frames follow the message schema described above."),
        (status = 404, description = "Resource not found; no body.")
    )
)]
pub async fn public_remote_control() {}

pub fn swagger_html(spec: &str) -> String {
    format!(
        r##"<!doctype html><html><head><title>Animeitor API</title><link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist/swagger-ui.css"></head><body><div id="swagger-ui"></div><script src="https://unpkg.com/swagger-ui-dist/swagger-ui-bundle.js"></script><script>SwaggerUIBundle({{url:"{spec}",dom_id:"#swagger-ui"}})</script></body></html>"##
    )
}
