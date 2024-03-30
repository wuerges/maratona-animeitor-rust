use warp::{filters::BoxedFilter, reply::Reply, Filter};

#[tracing::instrument(skip(into_iter))]
pub fn or_many<I, R>(into_iter: I) -> BoxedFilter<(R,)>
where
    I: IntoIterator<Item = BoxedFilter<(R,)>>,
    R: Reply + 'static,
{
    let mut iter = into_iter.into_iter();
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

pub trait OrMany<R> {
    fn collect_or(self) -> BoxedFilter<(R,)>
    where
        R: Reply;
}

impl<I, R> OrMany<R> for I
where
    I: IntoIterator<Item = BoxedFilter<(R,)>>,
    R: Reply + 'static,
{
    fn collect_or(self) -> BoxedFilter<(R,)>
    where
        R: Reply,
    {
        or_many::<I, R>(self)
    }
}
