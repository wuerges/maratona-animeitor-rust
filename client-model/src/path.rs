//! Parsing of the UI path `/animeitor/{event}/{contest}`.
//!
//! Platform-neutral (no DOM access): the wasm layer feeds it the pathname
//! segments; this module stays testable natively.

/// The event and contest a client is showing, from the UI path.
#[derive(PartialEq, Eq, Clone, Default, Debug)]
pub struct EventContest {
    pub event: String,
    pub contest: String,
}

/// Finds the event/contest after an `"animeitor"` segment.
///
/// The contest segment may be absent (the default contest `""`).
pub fn event_contest_from_segments(segments: &[&str]) -> Option<EventContest> {
    let pos = segments.iter().position(|s| *s == "animeitor")?;
    let event = segments.get(pos + 1)?.to_string();
    if event.is_empty() {
        return None;
    }
    Some(EventContest {
        event,
        contest: segments.get(pos + 2).unwrap_or(&"").to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn segments(pathname: &str) -> Vec<&str> {
        pathname.split('/').filter(|s| !s.is_empty()).collect()
    }

    fn parse(pathname: &str) -> Option<EventContest> {
        event_contest_from_segments(&segments(pathname))
    }

    #[test]
    fn event_without_contest_segment_is_the_default_contest() {
        assert_eq!(
            parse("/animeitor/regional-2026/"),
            Some(EventContest {
                event: "regional-2026".into(),
                contest: String::new(),
            })
        );
    }

    #[test]
    fn event_with_contest() {
        assert_eq!(
            parse("/animeitor/regional-2026/brasil"),
            Some(EventContest {
                event: "regional-2026".into(),
                contest: "brasil".into(),
            })
        );
    }

    #[test]
    fn extra_trailing_segments_are_ignored() {
        assert_eq!(
            parse("/animeitor/regional-2026/brasil/qualquer-coisa"),
            Some(EventContest {
                event: "regional-2026".into(),
                contest: "brasil".into(),
            })
        );
    }

    #[test]
    fn paths_without_animeitor_are_not_contests() {
        assert_eq!(parse("/"), None);
        assert_eq!(parse("/animeitor/"), None);
        assert_eq!(parse("/outra/coisa"), None);
    }
}
