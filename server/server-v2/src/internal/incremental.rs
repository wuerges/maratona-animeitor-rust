//! HTTP adapters for atomic store operations.
use super::*;
use axum::extract::{Query, rejection::QueryRejection};
use data::incremental::*;

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct KeepRuns {
    #[serde(default)]
    keep_runs: bool,
}
fn query(value: Result<Query<KeepRuns>, QueryRejection>) -> Result<bool, Response> {
    value.map(|Query(q)| q.keep_runs).map_err(|_| {
        error_json(
            StatusCode::BAD_REQUEST,
            "invalid_value",
            "keep_runs must be true or false",
        )
    })
}
fn body<T>(value: Result<Json<T>, JsonRejection>) -> Result<T, Response> {
    value.map(|Json(v)| v).map_err(map_json_rejection)
}

pub(super) async fn patch_event(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path(event): Path<String>,
    options: Result<Query<KeepRuns>, QueryRejection>,
    payload: Result<Json<EventPatch>, JsonRejection>,
) -> Result<Response, Response> {
    let result = store
        .patch_event(&event, body(payload)?, query(options)?)
        .await
        .map_err(store_error)?;
    Ok(data_json(result, StatusCode::OK))
}

pub(super) async fn patch_contest(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event, contest)): Path<(String, String)>,
    payload: Result<Json<ContestPatch>, JsonRejection>,
) -> Result<Response, Response> {
    let result = store
        .patch_contest(&event, &contest, body(payload)?)
        .await
        .map_err(store_error)?;
    Ok(data_json(result, StatusCode::OK))
}

pub(super) async fn patch_site(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event, contest, site)): Path<(String, String, String)>,
    payload: Result<Json<SitePatch>, JsonRejection>,
) -> Result<Response, Response> {
    let result = store
        .patch_site(&event, &contest, &site, body(payload)?)
        .await
        .map_err(store_error)?;
    Ok(data_json(result, StatusCode::OK))
}

pub(super) async fn add_team(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path(event): Path<String>,
    payload: Result<Json<NewTeam>, JsonRejection>,
) -> Result<Response, Response> {
    let result = store
        .add_team(&event, body(payload)?)
        .await
        .map_err(store_error)?;
    Ok(data_json(result, StatusCode::CREATED))
}

pub(super) async fn get_team(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event, login)): Path<(String, String)>,
) -> Result<Response, Response> {
    let result = store.get_team(&event, &login).await.map_err(store_error)?;
    Ok(data_json(result, StatusCode::OK))
}

pub(super) async fn patch_team(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event, login)): Path<(String, String)>,
    payload: Result<Json<TeamPatch>, JsonRejection>,
) -> Result<Response, Response> {
    let result = store
        .patch_team(&event, &login, body(payload)?)
        .await
        .map_err(store_error)?;
    Ok(data_json(result, StatusCode::OK))
}

pub(super) async fn remove_team(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event, login)): Path<(String, String)>,
    options: Result<Query<KeepRuns>, QueryRejection>,
) -> Result<Response, Response> {
    store
        .remove_team(&event, &login, query(options)?)
        .await
        .map_err(store_error)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub(super) async fn add_problem(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path(event): Path<String>,
    payload: Result<Json<NewProblem>, JsonRejection>,
) -> Result<Response, Response> {
    let result = store
        .add_problem(&event, body(payload)?)
        .await
        .map_err(store_error)?;
    Ok(data_json(result, StatusCode::CREATED))
}

pub(super) async fn remove_problem(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event, problem)): Path<(String, String)>,
) -> Result<Response, Response> {
    store
        .remove_problem(&event, &problem)
        .await
        .map_err(store_error)?;
    Ok(StatusCode::NO_CONTENT.into_response())
}

pub(super) async fn patch_contest_codes(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event, contest)): Path<(String, String)>,
    payload: Result<Json<CodesPatch>, JsonRejection>,
) -> Result<Response, Response> {
    let result = store
        .patch_contest_codes(&event, &contest, body(payload)?)
        .await
        .map_err(store_error)?;
    Ok(data_json(result, StatusCode::OK))
}

pub(super) async fn patch_site_codes(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event, contest, site)): Path<(String, String, String)>,
    payload: Result<Json<CodesPatch>, JsonRejection>,
) -> Result<Response, Response> {
    let result = store
        .patch_site_codes(&event, &contest, &site, body(payload)?)
        .await
        .map_err(store_error)?;
    Ok(data_json(result, StatusCode::OK))
}

pub(super) async fn get_contest(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event, contest)): Path<(String, String)>,
) -> Response {
    match store.get_contest(&event, &contest).await {
        Some(value) => data_json(value, StatusCode::OK),
        None => error_json(
            StatusCode::NOT_FOUND,
            "not_found",
            "event, contest or site does not exist",
        ),
    }
}

pub(super) async fn get_site(
    _auth: InternalAuth,
    State(store): State<EventStore>,
    Path((event, contest, site)): Path<(String, String, String)>,
) -> Response {
    match store.get_site(&event, &contest, &site).await {
        Some(value) => data_json(value, StatusCode::OK),
        None => error_json(
            StatusCode::NOT_FOUND,
            "not_found",
            "event, contest or site does not exist",
        ),
    }
}
