use seed::prelude::*;

use data::Team;

pub fn get_secret(url : &Url) -> String {
    url.search().get("secret").unwrap().first().unwrap().to_string()
}

pub fn get_url_filter(url : &Url) -> Option<Vec<String>> {
    url.search().get("filter").cloned()
}

pub fn get_sede(url : &Url) -> Option<String> {
    url.search().get("sede").unwrap_or(&vec![]).iter().cloned().next()
}

pub fn get_ws_url(path :&str) -> String {
    let window = web_sys::window().expect("Should have a window");
    let location = window.location().href().expect("Should have a URL");
    let url = web_sys::Url::new(&location)
        .expect("Location should be valid");
    url.set_protocol("ws:");
    url.set_pathname(path);
    url.href()

}


pub fn get_answer_hue_deg(num_problems: usize, problem_number: u32) -> u32 {
    (360 / num_problems as u32) * problem_number
}