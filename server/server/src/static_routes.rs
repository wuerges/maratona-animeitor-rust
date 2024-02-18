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

fn serve_volume(Volume { folder, path }: Volume) -> BoxedFilter<(File,)> {
    if path.is_empty() {
        warp::fs::dir(folder).boxed()
    } else {
        warp::path(path).and(warp::fs::dir(folder)).boxed()
    }
}
