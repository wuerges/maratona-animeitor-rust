use seed::prelude::*;

pub fn get_secret(url : &Url) -> String {
    url.search()
        .get("secret").expect("Error: no secret search field in URL")
        .first().expect("Error: secret param was empty")
        .to_string()
}

pub fn get_url_filter(url : &Url) -> Option<Vec<String>> {
    url.search().get("filter").cloned()
}

pub fn get_sede(url : &Url) -> Option<String> {
    url.search().get("sede").unwrap_or(&vec![]).iter().cloned().next()
}