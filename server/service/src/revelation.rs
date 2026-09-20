//! Frontend URL construction shared by the API and offline `printurls` CLI.
use url::Url;

/// The configured public origin hosts the frontend at `/animeitor/`.
pub fn scoreboard_url(public_url: &Url, event: &str, contest: &str) -> Url {
    let mut url = public_url.clone();
    url.set_query(None);
    url.set_fragment(None);
    url.path_segments_mut()
        .expect("validated public_url must support paths")
        .clear()
        .extend(["animeitor", event, contest, ""]);
    url
}

/// Add the frontend's revelation key and site selection query parameters.
pub fn revelation_url(scoreboard: &Url, site: &str, key: &str) -> Url {
    let mut url = scoreboard.clone();
    url.query_pairs_mut()
        .append_pair("secret", key)
        .append_pair("sede", site);
    url
}
