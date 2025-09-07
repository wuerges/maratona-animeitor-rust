pub struct Args {
    pub port: u16,
    pub server_api_key: String,
}

// pub async fn serve(
//     AppConfig {
//         config,
//         boca_url,
//         server_config: HttpConfig { port },
//         volumes,
//         server_api_key,
//     }: AppConfig,
// ) -> ServiceResult<()> {
//     let default_timeout = Duration::from_secs(3);

//     HttpServer::new(move || {
//         App::new()
//             .wrap(TracingLogger::default())
//             .wrap(Cors::permissive())
//             .app_data(web::Data::new(AppData {
//                 shared_db: shared_db.clone(),
//                 runs_tx: runs_tx.clone(),
//                 time_tx: time_tx.clone(),
//                 config: config.clone(),
//                 remote_control: remote_control.clone(),
//                 server_api_key: server_api_key.clone(),
//                 app_v2: Arc::new(AppV2::new(default_timeout)),
//             }))
//             .service(
//                 web::scope("api")
//                     .configure(api::configure)
//                     .service(get_metrics)
//                     .service(remote_control_ws),
//             )
//             .service(configure_volumes(volumes.clone()))
//     })
//     .bind(("0.0.0.0", port))?
//     .run()
//     .await?;

//     Ok(())
// }
