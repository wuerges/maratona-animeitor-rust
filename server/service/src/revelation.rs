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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event_store::{ContestConfig, EventState, EventStore, SiteConfig};

    #[tokio::test]
    async fn listing_is_sorted_encoded_and_matches_offline_url_builder() {
        let store = EventStore::with_revelation_salt("test-private-secret".into());
        let origin: Url = "https://public.example:8443/old/base".parse().unwrap();
        let event = "regional 2026 & ação";
        store
            .create_event(
                event,
                EventState {
                    name: event.into(),
                    problems: vec!["A".into()],
                    teams: vec![],
                    score_freeze_time_seconds: 14400,
                    penalty_seconds: 1200,
                    time_seconds: -60,
                    salt: Some("event-salt".into()),
                },
            )
            .await
            .unwrap();
        for contest in ["z brasil", "a méxico"] {
            store
                .create_contest(
                    event,
                    contest,
                    ContestConfig {
                        name: contest.into(),
                        codes: vec![".*".into()],
                        salt: Some("contest-salt".into()),
                        style: None,
                        ouro: 1,
                        prata: 2,
                        bronze: 3,
                        photo_url_format: None,
                        sound_url_format: None,
                    },
                )
                .await
                .unwrap();
            for site in ["z & sede?", "a sede/á"] {
                store
                    .create_site(
                        event,
                        contest,
                        site,
                        SiteConfig {
                            name: site.into(),
                            codes: vec![".*".into()],
                            salt: Some("site-salt".into()),
                        },
                    )
                    .await
                    .unwrap();
            }
        }
        let urls = store.revelation_urls(event, &origin).await.unwrap();
        let names: Vec<_> = urls
            .iter()
            .map(|row| (row.contest.as_str(), row.site.as_str()))
            .collect();
        assert_eq!(
            names,
            [
                ("a méxico", "a sede/á"),
                ("a méxico", "z & sede?"),
                ("z brasil", "a sede/á"),
                ("z brasil", "z & sede?")
            ]
        );
        for row in &urls {
            let key = crate::event_store::deployment_site_key(
                "test-private-secret",
                event,
                &row.contest,
                &row.site,
                "event-salt",
                "contest-salt",
                "site-salt",
            );
            let scoreboard = scoreboard_url(&origin, event, &row.contest);
            assert_eq!(
                row.url,
                revelation_url(&scoreboard, &row.site, &key).as_str()
            );
            let url: Url = row.url.parse().unwrap();
            assert_eq!(url.host_str(), Some("public.example"));
            assert_eq!(url.port(), Some(8443));
            assert!(!url.path().contains("old/base"));
            assert!(url.path().contains("%20"));
            let pairs: Vec<_> = url.query_pairs().into_owned().collect();
            assert_eq!(
                pairs,
                [
                    ("secret".into(), key.clone()),
                    ("sede".into(), row.site.clone())
                ]
            );
            assert_eq!(
                store
                    .site_by_key(event, &row.contest, &key)
                    .await
                    .unwrap()
                    .0,
                row.site
            );
        }
        // Changing a single site affects only its link.
        store
            .set_site_salt(event, "a méxico", "a sede/á", Some("new-site".into()))
            .await
            .unwrap();
        let site_changed = store.revelation_urls(event, &origin).await.unwrap();
        assert_ne!(site_changed[0].url, urls[0].url);
        assert_eq!(&site_changed[1..], &urls[1..]);
        // A contest rotation affects just its two sites.
        store
            .set_contest_salt(event, "a méxico", Some("new-contest".into()))
            .await
            .unwrap();
        let contest_changed = store.revelation_urls(event, &origin).await.unwrap();
        assert_ne!(contest_changed[0].url, site_changed[0].url);
        assert_ne!(contest_changed[1].url, site_changed[1].url);
        assert_eq!(&contest_changed[2..], &site_changed[2..]);
        store
            .set_event_salt(event, Some("new-event".into()))
            .await
            .unwrap();
        let event_changed = store.revelation_urls(event, &origin).await.unwrap();
        assert!(
            event_changed
                .iter()
                .zip(&contest_changed)
                .all(|(a, b)| a.url != b.url)
        );
    }
}
