use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use derivative::Derivative;
use regex::RegexSet;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::Team;

#[derive(Debug, Clone, Default, Derivative, ToSchema)]
#[derivative(PartialEq, Eq)]
pub struct RegexSetField(Vec<String>, #[derivative(PartialEq = "ignore")] RegexSet);

impl<'de> Deserialize<'de> for RegexSetField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let v = Vec::<String>::deserialize(deserializer)?;

        let automata = RegexSet::new(v.clone()).map_err(D::Error::custom)?;

        Ok(RegexSetField(v, automata))
    }
}

impl Serialize for RegexSetField {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl Display for RegexSetField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, ToSchema, PartialEq, Eq)]
/// A site entry.
pub struct SedeEntry {
    /// Site name.
    pub name: String,
    /// Site codes, using in filtering groups of sites.
    pub codes: RegexSetField,
    /// Style of the site (For CSS)
    pub style: Option<String>,
    /// Golden medal position.
    #[serde(default = "one")]
    pub ouro: usize,
    /// Silver medal position.
    #[serde(default = "two")]
    pub prata: usize,
    /// Bronze medal position.
    #[serde(default = "three")]
    pub bronze: usize,
}

fn one() -> usize {
    1
}
fn two() -> usize {
    2
}
fn three() -> usize {
    3
}

#[derive(Debug, Clone)]
pub struct Sede {
    pub entry: SedeEntry,
    automata: RegexSet,
}

impl PartialEq for Sede {
    fn eq(&self, other: &Self) -> bool {
        self.entry == other.entry
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
pub enum Color {
    Red,
    Gold,
    Silver,
    Bronze,
    Green,
    Yellow,
}

impl Sede {
    pub fn team_belongs(&self, team: &Team) -> bool {
        self.team_belongs_str(&team.login)
    }

    pub fn team_belongs_str(&self, team_login: &str) -> bool {
        self.automata.is_match(team_login)
    }

    pub fn premio(&self, p: usize) -> Option<Color> {
        if p <= self.entry.ouro {
            Some(Color::Gold)
        } else if p <= self.entry.prata {
            Some(Color::Silver)
        } else if p <= self.entry.bronze {
            Some(Color::Bronze)
        } else {
            None
        }
    }
}

impl SedeEntry {
    pub fn into_sede(&self) -> Sede {
        Sede {
            entry: self.clone(),
            automata: self.codes.1.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
/// Site configuration for contest.
pub struct ConfigContest {
    /// The contest title.
    pub titulo: SedeEntry,
    /// A site entry.
    pub sedes: Option<Vec<SedeEntry>>,
}

impl ConfigContest {
    pub fn into_contest(&self) -> Contest {
        let entry_map: HashMap<String, SedeEntry> = self
            .sedes
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|sede| (sede.name.clone(), sede))
            .collect();

        Contest {
            titulo: self.titulo.into_sede(),
            sedes: entry_map
                .iter()
                .map(|(name, entry)| (name.clone(), entry.into_sede()))
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct Contest {
    pub sedes: HashMap<String, Sede>,
    pub titulo: Sede,
}

impl Contest {
    pub fn get_sede_nome_sede(&self, name: &str) -> Option<&Sede> {
        if self.titulo.entry.name == name {
            return Some(&self.titulo);
        }
        self.sedes.get(name)
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SedeSecret {
    pub name: String,
    pub secret: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ConfigSecret {
    pub secrets: Vec<SedeSecret>,
}

#[derive(Debug)]
pub struct Secret {
    /// A map where the key is a shared secret, and the sede is a contest site
    pub sedes_by_secret: HashMap<String, Sede>,
}

impl Secret {
    pub fn get_sede_by_secret(&self, key: &str) -> Option<&Sede> {
        self.sedes_by_secret.get(key)
    }
}

impl ConfigSecret {
    pub fn into_secret(&self, salt: Option<String>, sedes: &Contest) -> Secret {
        let salt = salt.unwrap_or_default();
        let sedes_by_secret = self
            .secrets
            .iter()
            .filter_map(|sede_secret| {
                let complete = format!("{}{}", salt, &sede_secret.secret);
                sedes
                    .get_sede_nome_sede(&sede_secret.name)
                    .map(|sede| (complete, sede.clone()))
            })
            .collect();
        Secret { sedes_by_secret }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_config_patterns() {
        let sede = SedeEntry {
            name: "sede-name".into(),
            codes: serde_json::from_value(json!(["teambr", "teammx"])).unwrap(),
            ..SedeEntry::default()
        };

        let config_contest = ConfigContest {
            sedes: Some(vec![sede]),
            titulo: SedeEntry {
                name: "dummy".to_string(),
                ..SedeEntry::default()
            },
        };
        let contest = config_contest.into_contest();

        let config_secret = ConfigSecret {
            secrets: vec![SedeSecret {
                name: "sede-name".into(),
                secret: "key".into(),
            }],
        };
        let secret = config_secret.into_secret(None, &contest);

        assert!(
            secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("teambr$"),
        );
        assert!(
            secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("teammx$")
        );
        assert!(
            secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("$teammx$")
        );
        assert!(
            secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("$teammx$")
        );
        assert!(
            secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("$teammx")
        );
        assert!(
            secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("$teammx")
        );

        assert!(
            !secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("tea#mbr$")
        );
        assert!(
            !secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("tea#mmx$")
        );
        assert!(
            !secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("$te#ammx$")
        );
        assert!(
            !secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("$te#ammx$")
        );
        assert!(
            !secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("$te#ammx")
        );
        assert!(
            !secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("$te#ammx")
        );

        assert!(
            !secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("teamag")
        );
        assert!(
            !secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("teamag$")
        );
        assert!(
            !secret
                .get_sede_by_secret("key")
                .unwrap()
                .team_belongs_str("$teamag$")
        );
    }
}
