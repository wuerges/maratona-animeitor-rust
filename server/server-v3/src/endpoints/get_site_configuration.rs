use actix_web::{HttpResponse, get, web};
use tracing::instrument;

use crate::{components::rejection::NotFound, model::app::AppV2};

#[instrument(skip_all)]
#[get("/contests/{contest}/sites")]
pub async fn get_site_configuration(
    data: web::Data<AppV2>,
    contest: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {
    let contest = data.get_contest(&contest).await.ok_or(NotFound)?;

    let sites = contest.get_site_configuration().await.ok_or(NotFound)?;

    Ok(HttpResponse::Ok().json(sites))
}
