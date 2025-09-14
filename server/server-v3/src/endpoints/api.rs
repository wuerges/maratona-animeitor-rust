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
))]
struct ApiDoc;

#[instrument(skip_all)]
#[get("/openapi.json")]
pub async fn open_api() -> Result<String, serde_json::Error> {
    ApiDoc::openapi().to_pretty_json()
}
