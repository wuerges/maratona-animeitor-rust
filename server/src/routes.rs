use crate::dataio::DB;

use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;


pub fn with_db(
    db: Arc<Mutex<DB>>,
) -> impl Filter<Extract = (Arc<Mutex<DB>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}