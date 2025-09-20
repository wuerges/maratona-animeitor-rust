use actix_web::{get, web};
use sdk::ContestParameters;

use crate::{
    components::{rejection::NotFoundContest, success::Data},
    model::app::AppV2,
};

#[utoipa::path(responses(
    NotFoundContest,
    Data<ContestParameters>
))]
#[get("/contests/{contest}/parameters")]
pub async fn get_contest_parameters(
    data: web::Data<AppV2>,
    contest: web::Path<String>,
) -> Result<Data<ContestParameters>, actix_web::Error> {
    let contest = data.get_contest(&contest).await.ok_or(NotFoundContest)?;

    Ok(Data::new(contest.get_parameters().await))
}
