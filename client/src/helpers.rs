use seed::prelude::*;

use data::Team;

pub fn get_secret(url : &Url) -> String {
    url.search().get("secret").unwrap().first().unwrap().to_string()
}

pub fn get_url_filter(url : &Url) -> Option<Vec<String>> {
    url.search().get("filter").cloned()
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

pub fn check_filter_login(url_filter: &Option<Vec<String>>, t : &String) -> bool {
    match url_filter {
        None => true,
        Some(tot) => {
            for f in tot {
                if t.find(f).is_some() {
                    return true
                }
            }
            return false
        },
    }
}

pub fn check_filter(url_filter: &Option<Vec<String>>, t : &Team) -> bool {
    check_filter_login(url_filter, &t.login)
}
