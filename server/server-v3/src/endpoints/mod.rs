use actix_web::web;

pub mod contest_admin;
mod get_contest_runs;
mod get_contest_time;

pub fn as_service(service_config: &mut web::ServiceConfig) {
    service_config.service((
        get_contest_runs::get_contest_runs,
        get_contest_time::get_contest_time,
    ));
}
