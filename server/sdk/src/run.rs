use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

#[derive(
    Debug,
    SerializeDisplay,
    DeserializeFromStr,
    Clone,
    Copy,
    strum::Display,
    strum::EnumString,
    PartialEq,
    Eq,
)]
#[strum(serialize_all = "snake_case")]
pub enum Answer {
    Yes,
    No,
    Undecided,
    NoWithoutPenalty,
}

/// A run, modeled from a webcast run
/// 361626341239teammxmx12E?
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Run {
    pub id: u64,
    pub time_in_seconds: u64,
    pub team_login: String,
    pub problem_letter: String,
    pub answer: Answer,
}

#[cfg(test)]
mod tests {

    use super::*;
    use assert2::let_assert;
    use rstest::rstest;

    #[rstest]
    #[case(
        Run {
            id: 1,
            time_in_seconds: 255,
            team_login: "teambrsc001".to_string(),
            problem_letter: "A".to_string(),
            answer: Answer::Yes
        },
        serde_json::json!({
            "id": 1,
            "time_in_seconds": 255,
            "team_login": "teambrsc001",
            "problem_letter": "A",
            "answer": "yes"
        })
    )]
    fn check_serialize(#[case] run: Run, #[case] expected: serde_json::Value) {
        use assert2::check;

        let_assert!(Ok(serialized) = serde_json::to_value(&run));

        check!(serialized == expected);

        let_assert!(Ok(deserialized) = serde_json::from_value::<Run>(expected));

        check!(deserialized == run);
    }
}
