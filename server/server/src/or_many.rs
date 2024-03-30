use warp::{filters::BoxedFilter, reply::Reply, Filter};

#[tracing::instrument(skip(into_iter))]
pub fn or_many<F, I, T, R>(into_iter: I) -> BoxedFilter<(R,)>
where
    F: Filter + Send + Sync,
    F::Extract: Send,
    I: IntoIterator<Item = BoxedFilter<(R,)>>,
    R: Reply,
{
    let iter = into_iter.into_iter();
    let first = iter.next();

    match first {
        Some(first) => iter
            .fold(first.boxed(), |acc, next| acc.or(next).unify().boxed())
            .boxed(),

        None => warp::any()
            .and_then(|| async { Err(warp::reject::not_found()) })
            .boxed(),
    }
}
