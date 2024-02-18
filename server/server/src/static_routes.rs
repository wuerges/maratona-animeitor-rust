use service::volume::Volume;
use warp::{
    filters::{fs::File, BoxedFilter},
    Filter,
};

#[tracing::instrument]
pub fn serve_static_routes(mut volumes: Vec<Volume>) -> BoxedFilter<(File,)> {
    tracing::info!(?volumes, "serving volumes");

    let last = volumes.pop();

    match last {
        Some(last) => {
            let last = serve_volume(last);

            volumes
                .into_iter()
                .fold(last, |acc, volume| {
                    acc.or(serve_volume(volume)).unify().boxed()
                })
                .boxed()
        }

        None => warp::any()
            .and_then(|| async { Err(warp::reject::not_found()) })
            .boxed(),
    }
}

fn serve_volume(volume: Volume) -> BoxedFilter<(File,)> {
    warp::path(volume.path)
        .and(warp::fs::dir(volume.folder))
        .boxed()
}
