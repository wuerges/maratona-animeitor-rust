use regex_set_field::RegexSetField;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq, ToSchema)]
/// A site entry.
pub struct Site {
    /// Site name.
    pub name: String,

    /// Site codes, using in filtering groups of sites.
    #[schema(value_type = Vec<String>)]
    pub codes: RegexSetField,

    /// Golden medal position.
    #[serde(default = "one", alias = "ouro")]
    pub gold: u32,

    /// Silver medal position.
    #[serde(default = "two", alias = "prata")]
    pub silver: u32,
    /// Bronze medal position.
    #[serde(default = "three")]
    pub bronze: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
/// Site configuration for contest.
pub struct SiteConfiguration {
    /// The contest base.
    #[serde(alias = "titulo")]
    pub base: Site,
    /// A site entry.
    #[serde(alias = "sedes", default)]
    pub sites: Vec<Site>,
}

fn one() -> u32 {
    1
}

fn two() -> u32 {
    2
}

fn three() -> u32 {
    3
}

impl Site {
    pub fn team_belongs(&self, team_login: &str) -> bool {
        self.codes.as_regex_set().is_match(team_login)
    }
}
