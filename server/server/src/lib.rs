mod dbupdate;
mod errors;
pub mod metrics;
pub mod openapi;
pub mod or_many;
mod runs;
mod secret;
mod server;
mod static_routes;
mod timer;

pub use self::server::serve_simple_contest;
