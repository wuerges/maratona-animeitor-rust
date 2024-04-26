use service::volume::Volume;

pub fn configure_volumes(volumes: Vec<Volume>) -> Vec<actix_files::Files> {
    volumes
        .into_iter()
        .map(|Volume { folder, path }| actix_files::Files::new(&path, &folder).show_files_listing())
        .collect()
}
