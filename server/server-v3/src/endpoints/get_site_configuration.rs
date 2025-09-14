use actix_web::{get, web};
use sdk::SiteConfiguration;
use tracing::instrument;

use crate::{
    components::{rejection::NotFound, success::Data},
    model::app::AppV2,
};

#[instrument(skip_all)]
#[get("/contests/{contest}/sites")]
pub async fn get_site_configuration(
    data: web::Data<AppV2>,
    contest: web::Path<String>,
) -> Result<Data<SiteConfiguration>, NotFound> {
    let contest = data.get_contest(&contest).await.ok_or(NotFound)?;

    let sites = contest.get_site_configuration().await.ok_or(NotFound)?;

    Ok(Data::new(sites))
}
