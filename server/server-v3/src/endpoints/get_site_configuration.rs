use actix_web::{get, web};
use sdk::SiteConfiguration;
use tracing::instrument;

use crate::{
    components::{
        rejection::{NotFoundContest, NotFoundSite},
        success::Data,
    },
    model::app::AppV2,
};

#[utoipa::path(
    responses(
        NotFoundContest,
        NotFoundSite,
        Data<SiteConfiguration>
    ),
)]
#[instrument(skip_all)]
#[get("/contests/{contest}/sites")]
pub async fn get_site_configuration(
    data: web::Data<AppV2>,
    contest: web::Path<String>,
) -> Result<Data<SiteConfiguration>, actix_web::Error> {
    let contest = data.get_contest(&contest).await.ok_or(NotFoundContest)?;

    let sites = contest.get_site_configuration().await.ok_or(NotFoundSite)?;

    Ok(Data::new(sites))
}
