use crate::errors::ServiceResult;
use crate::webcast;

/// Polls BOCA (webcast) and feeds the event store: the legacy `-i` mode,
/// now publishing into the default event.
pub async fn store_update_loop(
    boca_url: String,
    store: crate::event_store::EventStore,
    event_name: String,
) -> ServiceResult<()> {
    let dur = tokio::time::Duration::new(1, 0);
    let mut interval = tokio::time::interval(dur);
    loop {
        interval.tick().await;

        match webcast::load_data_from_url_maybe(&boca_url).await {
            Ok(contest_state) => {
                let (event, runs) =
                    crate::event_store::from_legacy_contest_state(&contest_state, &event_name);
                let result = store.upsert_event_with_runs(&event_name, event, runs).await;
                match result {
                    Ok(()) => (),
                    Err(error) => {
                        tracing::error!("Retrying after error updating event: \n{}", error)
                    }
                }
            }
            Err(error) => {
                tracing::error!("Retrying after error loading data: \n{}", error);
            }
        }
    }
}

