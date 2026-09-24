//! Contracts for incremental management operations.
#![allow(dead_code)] // Annotation-only handlers used to generate the served specification.
use super::*;
use data::incremental::*;

#[derive(OpenApi)]
#[openapi(paths(
    patch_event,
    get_contest,
    patch_contest,
    get_site,
    patch_site,
    add_team,
    get_team,
    patch_team,
    remove_team,
    add_problem,
    remove_problem,
    patch_contest_codes,
    patch_site_codes
))]
pub(super) struct IncrementalApiDoc;

/// Patch event
///
/// Change selected event fields atomically, including photo and sound URL templates shared by all contests. Omitted fields remain unchanged, arrays replace whole lists, and null clears salt. Removing teams with stored runs requires keep_runs=true; removing referenced problems is always a conflict. Event name cannot change. Validation failure changes nothing. Unlike legacy PUT, this endpoint checks references.
#[utoipa::path(patch, path = "/internal/events/{event_name}", operation_id = "incremental_patch_event", tag = "Incremental management",
    params(("event_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment"), ("keep_runs" = Option<bool>, Query, description = "Default false. Explicitly retain runs when removing teams; never overrides problem reference checks")),
    request_body(content = EventPatch, example = json!({"penalty_seconds": 600})),
    responses(
        (status = 200, description = "Complete updated resource in data", body = Success<EventState>),
        (status = 400, description = "Invalid input; unknown fields, null required fields, empty patches, and invalid regexes are rejected", body = Failure),
        (status = 401, description = "Missing or invalid Basic credentials", body = Failure),
        (status = 404, description = "Event, parent resource, or item does not exist", body = Failure),
        (status = 409, description = "Duplicate identifier, ambiguous legacy collection, or removal of an item with stored runs", body = Failure),
    )
)]
async fn patch_event() {}

/// Get contest
///
/// Read a complete contest configuration, including salts, before or after event start.
#[utoipa::path(get, path = "/internal/contests/{event_name}/{contest_name}", operation_id = "incremental_get_contest", tag = "Incremental management",
    params(("event_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment"), ("contest_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment")),
    responses(
        (status = 200, description = "Complete updated resource in data", body = Success<ContestConfig>),
        (status = 400, description = "Invalid input; unknown fields, null required fields, empty patches, and invalid regexes are rejected", body = Failure),
        (status = 401, description = "Missing or invalid Basic credentials", body = Failure),
        (status = 404, description = "Event, parent resource, or item does not exist", body = Failure),
        (status = 409, description = "Duplicate identifier, ambiguous legacy collection, or removal of an item with stored runs", body = Failure),
    )
)]
async fn get_contest() {}

/// Patch contest
///
/// Change selected contest fields atomically. Omitted fields remain unchanged; arrays replace whole lists. Null clears salt or style. Media templates belong to the event. Name cannot change; sites are preserved. Invalid regexes leave both configuration and compiled filters unchanged.
#[utoipa::path(patch, path = "/internal/contests/{event_name}/{contest_name}", operation_id = "incremental_patch_contest", tag = "Incremental management",
    params(("event_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment"), ("contest_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment")),
    request_body(content = ContestPatch, example = json!({"ouro": 4, "style": null})),
    responses(
        (status = 200, description = "Complete updated resource in data", body = Success<ContestConfig>),
        (status = 400, description = "Invalid input; unknown fields, null required fields, empty patches, and invalid regexes are rejected", body = Failure),
        (status = 401, description = "Missing or invalid Basic credentials", body = Failure),
        (status = 404, description = "Event, parent resource, or item does not exist", body = Failure),
        (status = 409, description = "Duplicate identifier, ambiguous legacy collection, or removal of an item with stored runs", body = Failure),
    )
)]
async fn patch_contest() {}

/// Get site
///
/// Read a complete site configuration, including its salt, before or after event start.
#[utoipa::path(get, path = "/internal/sites/{event_name}/{contest_name}/{site_name}", operation_id = "incremental_get_site", tag = "Incremental management",
    params(("event_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment"), ("contest_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment"), ("site_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment")),
    responses(
        (status = 200, description = "Complete updated resource in data", body = Success<SiteConfig>),
        (status = 400, description = "Invalid input; unknown fields, null required fields, empty patches, and invalid regexes are rejected", body = Failure),
        (status = 401, description = "Missing or invalid Basic credentials", body = Failure),
        (status = 404, description = "Event, parent resource, or item does not exist", body = Failure),
        (status = 409, description = "Duplicate identifier, ambiguous legacy collection, or removal of an item with stored runs", body = Failure),
    )
)]
async fn get_site() {}

/// Patch site
///
/// Change selected site fields atomically. Omitted fields remain unchanged; codes replaces the complete filter list. Null clears salt. Name cannot change. Reconnect run streams when changing their contest filters.
#[utoipa::path(patch, path = "/internal/sites/{event_name}/{contest_name}/{site_name}", operation_id = "incremental_patch_site", tag = "Incremental management",
    params(("event_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment"), ("contest_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment"), ("site_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment")),
    request_body(content = SitePatch, example = json!({"salt": null})),
    responses(
        (status = 200, description = "Complete updated resource in data", body = Success<SiteConfig>),
        (status = 400, description = "Invalid input; unknown fields, null required fields, empty patches, and invalid regexes are rejected", body = Failure),
        (status = 401, description = "Missing or invalid Basic credentials", body = Failure),
        (status = 404, description = "Event, parent resource, or item does not exist", body = Failure),
        (status = 409, description = "Duplicate identifier, ambiguous legacy collection, or removal of an item with stored runs", body = Failure),
    )
)]
async fn patch_site() {}

/// Add team
///
/// Append one team to the event roster. A duplicate login returns 409. Legacy duplicate logins make item operations ambiguous and must be repaired by replacing the roster.
#[utoipa::path(post, path = "/internal/events/{event_name}/teams", operation_id = "incremental_add_team", tag = "Incremental management",
    params(("event_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment")),
    request_body(content = NewTeam, example = json!({"login": "teambr002", "escola": "Example University", "nome": "Second Team"})),
    responses(
        (status = 201, description = "Complete updated resource in data", body = Success<TeamInfo>),
        (status = 400, description = "Invalid input; unknown fields, null required fields, empty patches, and invalid regexes are rejected", body = Failure),
        (status = 401, description = "Missing or invalid Basic credentials", body = Failure),
        (status = 404, description = "Event, parent resource, or item does not exist", body = Failure),
        (status = 409, description = "Duplicate identifier, ambiguous legacy collection, or removal of an item with stored runs", body = Failure),
    )
)]
async fn add_team() {}

/// Get team
///
/// Read one team by its exact login. Missing teams return 404; duplicate legacy logins return 409.
#[utoipa::path(get, path = "/internal/events/{event_name}/teams/{login}", operation_id = "incremental_get_team", tag = "Incremental management",
    params(("event_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment"), ("login" = String, Path, description = "Exact resource identifier; URL-encode as one path segment")),
    responses(
        (status = 200, description = "Complete updated resource in data", body = Success<TeamInfo>),
        (status = 400, description = "Invalid input; unknown fields, null required fields, empty patches, and invalid regexes are rejected", body = Failure),
        (status = 401, description = "Missing or invalid Basic credentials", body = Failure),
        (status = 404, description = "Event, parent resource, or item does not exist", body = Failure),
        (status = 409, description = "Duplicate identifier, ambiguous legacy collection, or removal of an item with stored runs", body = Failure),
    )
)]
async fn get_team() {}

/// Patch team
///
/// Change escola and/or nome atomically, preserving unspecified fields. Login is immutable. Empty patches and unknown fields are rejected. Legacy duplicate logins return 409.
#[utoipa::path(patch, path = "/internal/events/{event_name}/teams/{login}", operation_id = "incremental_patch_team", tag = "Incremental management",
    params(("event_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment"), ("login" = String, Path, description = "Exact resource identifier; URL-encode as one path segment")),
    request_body(content = TeamPatch, example = json!({"nome": "Updated Team"})),
    responses(
        (status = 200, description = "Complete updated resource in data", body = Success<TeamInfo>),
        (status = 400, description = "Invalid input; unknown fields, null required fields, empty patches, and invalid regexes are rejected", body = Failure),
        (status = 401, description = "Missing or invalid Basic credentials", body = Failure),
        (status = 404, description = "Event, parent resource, or item does not exist", body = Failure),
        (status = 409, description = "Duplicate identifier, ambiguous legacy collection, or removal of an item with stored runs", body = Failure),
    )
)]
async fn patch_team() {}

/// Remove team
///
/// Remove a team. Stored runs cause 409 unless keep_runs=true. That option retains submissions and replay history; regex streams may still include them. Recreating the login associates retained runs with it again. New ingestion for an absent team is skipped with unknown_team warnings. Legacy duplicate logins return 409.
#[utoipa::path(delete, path = "/internal/events/{event_name}/teams/{login}", operation_id = "incremental_remove_team", tag = "Incremental management",
    params(("event_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment"), ("login" = String, Path, description = "Exact resource identifier; URL-encode as one path segment"), ("keep_runs" = Option<bool>, Query, description = "Default false. Explicitly retain runs when removing teams; never overrides problem reference checks")),
    responses(
        (status = 204, description = "Deleted; no response body"),
        (status = 400, description = "Invalid input; unknown fields, null required fields, empty patches, and invalid regexes are rejected", body = Failure),
        (status = 401, description = "Missing or invalid Basic credentials", body = Failure),
        (status = 404, description = "Event, parent resource, or item does not exist", body = Failure),
        (status = 409, description = "Duplicate identifier, ambiguous legacy collection, or removal of an item with stored runs", body = Failure),
    )
)]
async fn remove_team() {}

/// Add problem
///
/// Append one problem identifier to the ordered event list. Duplicate identifiers return 409. Reorder using event PATCH with a complete problems array. Legacy duplicate identifiers must be repaired before item operations.
#[utoipa::path(post, path = "/internal/events/{event_name}/problems", operation_id = "incremental_add_problem", tag = "Incremental management",
    params(("event_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment")),
    request_body(content = NewProblem, example = json!({"problem": "C"})),
    responses(
        (status = 201, description = "Complete updated resource in data", body = Success<Vec<String>>),
        (status = 400, description = "Invalid input; unknown fields, null required fields, empty patches, and invalid regexes are rejected", body = Failure),
        (status = 401, description = "Missing or invalid Basic credentials", body = Failure),
        (status = 404, description = "Event, parent resource, or item does not exist", body = Failure),
        (status = 409, description = "Duplicate identifier, ambiguous legacy collection, or removal of an item with stored runs", body = Failure),
    )
)]
async fn add_problem() {}

/// Remove problem
///
/// Remove an unreferenced problem identifier. Stored runs or legacy duplicate identifiers cause 409. There is no keep_runs override for problems; clear runs first if removal is intended.
#[utoipa::path(delete, path = "/internal/events/{event_name}/problems/{problem}", operation_id = "incremental_remove_problem", tag = "Incremental management",
    params(("event_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment"), ("problem" = String, Path, description = "Exact resource identifier; URL-encode as one path segment")),
    responses(
        (status = 204, description = "Deleted; no response body"),
        (status = 400, description = "Invalid input; unknown fields, null required fields, empty patches, and invalid regexes are rejected", body = Failure),
        (status = 401, description = "Missing or invalid Basic credentials", body = Failure),
        (status = 404, description = "Event, parent resource, or item does not exist", body = Failure),
        (status = 409, description = "Duplicate identifier, ambiguous legacy collection, or removal of an item with stored runs", body = Failure),
    )
)]
async fn remove_problem() {}

/// Patch contest codes
///
/// Atomically add/remove exact regex strings. Retain existing order and append new patterns in request order. Existing additions and missing removals are no-ops; remove all exact duplicates. Reject overlap, empty operations, and invalid resulting regex sets without changing state. Reconnect public run streams after changing contest filters.
#[utoipa::path(patch, path = "/internal/contests/{event_name}/{contest_name}/codes", operation_id = "incremental_patch_contest_codes", tag = "Incremental management",
    params(("event_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment"), ("contest_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment")),
    request_body(content = CodesPatch, example = json!({"add": ["^teambr002$"], "remove": ["^teambr001$"]})),
    responses(
        (status = 200, description = "Complete updated resource in data", body = Success<ContestConfig>),
        (status = 400, description = "Invalid input; unknown fields, null required fields, empty patches, and invalid regexes are rejected", body = Failure),
        (status = 401, description = "Missing or invalid Basic credentials", body = Failure),
        (status = 404, description = "Event, parent resource, or item does not exist", body = Failure),
        (status = 409, description = "Duplicate identifier, ambiguous legacy collection, or removal of an item with stored runs", body = Failure),
    )
)]
async fn patch_contest_codes() {}

/// Patch site codes
///
/// Atomically add/remove exact site regex strings. Retain existing order and append new patterns in request order. Existing additions and missing removals are no-ops; remove all exact duplicates. Reject overlap, empty operations, and invalid resulting regex sets without changing state.
#[utoipa::path(patch, path = "/internal/sites/{event_name}/{contest_name}/{site_name}/codes", operation_id = "incremental_patch_site_codes", tag = "Incremental management",
    params(("event_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment"), ("contest_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment"), ("site_name" = String, Path, description = "Exact resource identifier; URL-encode as one path segment")),
    request_body(content = CodesPatch, example = json!({"add": ["^teambr002$"], "remove": ["^teambr001$"]})),
    responses(
        (status = 200, description = "Complete updated resource in data", body = Success<SiteConfig>),
        (status = 400, description = "Invalid input; unknown fields, null required fields, empty patches, and invalid regexes are rejected", body = Failure),
        (status = 401, description = "Missing or invalid Basic credentials", body = Failure),
        (status = 404, description = "Event, parent resource, or item does not exist", body = Failure),
        (status = 409, description = "Duplicate identifier, ambiguous legacy collection, or removal of an item with stored runs", body = Failure),
    )
)]
async fn patch_site_codes() {}
