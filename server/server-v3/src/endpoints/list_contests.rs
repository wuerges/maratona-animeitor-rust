use actix_web::{get, web};
use tracing::instrument;

use crate::{components::success::Data, model::app::AppV2};

/// List contests
#[utoipa::path(
    responses(Data<Vec<String>>),
)]
#[instrument(skip_all)]
#[get("/contests")]
pub async fn list_contests(data: web::Data<AppV2>) -> Data<Vec<String>> {
    let contests = data.list_contests().await;

    Data::new(contests)
}
