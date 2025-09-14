use actix_web::{HttpResponse, get, web};
use tracing::instrument;

use crate::model::app::AppV2;

#[instrument(skip_all)]
#[get("/contests")]
pub async fn list_contests(data: web::Data<AppV2>) -> Result<HttpResponse, actix_web::Error> {
    let contests = data.list_contests().await;

    Ok(HttpResponse::Ok().json(contests))
}
