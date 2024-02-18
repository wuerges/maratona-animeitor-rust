use itertools::Either;
use service::volume::Volume;
use warp::{
    filters::{fs::File, BoxedFilter},
    reply::{Reply, Response},
    Filter,
};

pub fn serve_static_routes(mut volumes: Vec<Volume>) -> BoxedFilter<(Response,)> {
    let base = warp::any()
        .map(|| "serving static content".to_string())
        .boxed();

    let last = volumes.pop();

    match last {
        Some(last) => serve_volume(last).map(|x: File| x.into_response()).boxed(),
        None => warp::any()
            .and_then(|| async { Err(warp::reject::not_found()) })
            .boxed(),
    }
}

fn serve_volume(volume: Volume) -> BoxedFilter<(File,)> {
    warp::path(volume.folder)
        .and(warp::fs::dir(volume.path))
        .boxed()
}
