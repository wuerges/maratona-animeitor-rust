use actix_web::{HttpRequest, HttpResponse, get, web};
use tokio::pin;
use tokio_stream::StreamExt;
use tracing::{debug, warn};

use crate::model::app::AppV2;

#[get("/contests/{contest}/runs")]
pub async fn get_contest_runs(
    data: web::Data<AppV2>,
    contest: web::Path<String>,
    body: web::Payload,
    req: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let contest = data.get_contest(&contest).await?;

    let (response, mut session, _msg_stream) = actix_ws::handle(&req, body)?;

    actix_web::rt::spawn(async move {
        let runs = contest.get_runs().await;
        pin!(runs);

        while let Some(batch) = runs.next().await {
            let batch = batch.await;

            for run in batch {
                match serde_json::to_string(&run) {
                    Err(err) => warn!(?err, ?run, "failed serializing run"),
                    Ok(run) => {
                        if let Err(err) = session.text(run).await {
                            debug!(?err, "websocket closed");
                            return;
                        }
                    }
                }
            }
        }
    });

    Ok(response)
}
