use derivative::Derivative;
use regex::RegexSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Derivative, Deserialize, Serialize)]
#[serde(try_from = "Vec<String>", into = "Vec<String>")]
#[derivative(PartialEq, Eq)]
pub struct RegexSetField(Vec<String>, #[derivative(PartialEq = "ignore")] RegexSet);

impl RegexSetField {
    pub fn as_strings(&self) -> &[String] {
        &self.0
    }

    pub fn as_regex_set(&self) -> &RegexSet {
        &self.1
    }
}

impl TryFrom<Vec<String>> for RegexSetField {
    type Error = regex::Error;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        let automata = RegexSet::new(value.clone())?;

        Ok(RegexSetField(value, automata))
    }
}

impl From<RegexSetField> for Vec<String> {
    fn from(value: RegexSetField) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use assert2::let_assert;
    use rstest::rstest;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Test {
        field: RegexSetField,
    }

    #[rstest]
    #[case(
        Test { field: vec!["a.*b".to_string(), "c.d".to_string()].try_into().unwrap() },
        serde_json::json!({ "field": ["a.*b", "c.d"]})
    )]
    fn check_serialize(#[case] run: Test, #[case] expected: serde_json::Value) {
        use assert2::check;

        let_assert!(Ok(serialized) = serde_json::to_value(&run));

        check!(serialized == expected);

        let_assert!(Ok(deserialized) = serde_json::from_value::<Test>(expected));

        check!(deserialized == run);
    }
}
