use actix_web::web;

pub mod contest_admin;

mod api;
mod get_contest_runs;
mod get_contest_time;
mod get_site_configuration;
mod list_contests;

pub fn as_service(service_config: &mut web::ServiceConfig) {
    service_config.service((
        get_contest_runs::get_contest_runs,
        get_contest_runs::get_contest_runs_unmasked,
        get_contest_time::get_contest_time,
        get_site_configuration::get_site_configuration,
        list_contests::list_contests,
        api::open_api,
    ));
}
