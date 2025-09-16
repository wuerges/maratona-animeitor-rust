use crate::endpoints;
use actix_web::get;
use tracing::instrument;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(paths(
    endpoints::get_contest_runs::get_contest_runs,
    endpoints::get_contest_runs::get_contest_runs_unmasked,
    endpoints::get_contest_time::get_contest_time,
    endpoints::get_site_configuration::get_site_configuration,
    endpoints::list_contests::list_contests,
    open_api
))]
struct ApiDoc;

#[instrument(skip_all)]
#[utoipa::path()]
#[get("/openapi.json")]
pub async fn open_api() -> Result<String, serde_json::Error> {
    ApiDoc::openapi().to_pretty_json()
}

#[derive(OpenApi)]
#[openapi(paths(
    endpoints::contest_admin::update_contest_parameters,
    endpoints::contest_admin::update_contest_sites,
    endpoints::contest_admin::update_contest_state,
    endpoints::contest_admin::create_contest,
    endpoints::contest_admin::update_contest_secret,
    open_api_internal
))]
struct InternalApiDoc;

#[instrument(skip_all)]
#[utoipa::path(context_path = "/internal")]
#[get("/openapi.json")]
pub async fn open_api_internal() -> Result<String, serde_json::Error> {
    let public = ApiDoc::openapi();
    let internal = InternalApiDoc::openapi().merge_from(public);

    internal.to_pretty_json()
}
