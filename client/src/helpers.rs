use seed::prelude::Url;


pub fn get_source(url : &Url) -> Option<String> {
    match url.search().get("source") {
        None => None,
        Some(v) => v.first().map(|s| s.to_string()),
    }
}

pub fn get_secret(url : &Url) -> String {
    url.search().get("secret").unwrap().first().unwrap().to_string()
}

pub fn get_url_filter(url : &Url) -> Option<Vec<String>> {
    url.search().get("filter").cloned()
}
