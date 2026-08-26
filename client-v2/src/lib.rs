mod api;

pub mod views;

pub fn init_config(config: client_sdk::SdkConfig) {
    api::init_config(config);
}
