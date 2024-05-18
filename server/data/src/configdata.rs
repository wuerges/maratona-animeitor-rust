use std::collections::HashMap;

use aho_corasick::AhoCorasick;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::Team;

#[derive(Deserialize, Serialize, Debug, Clone, Default, ToSchema, PartialEq, Eq)]
/// A site entry.
pub struct SedeEntry {
    /// Site name.
    pub name: String,
    /// Site codes, using in filtering groups of sites.
    pub codes: Vec<String>,
    /// Style of the site (For CSS)
    pub style: Option<String>,
    /// Golden medal position.
    pub ouro: Option<usize>,
    /// Silver medal position.
    pub prata: Option<usize>,
    /// Bronze medal position.
    pub bronze: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct Sede {
    pub entry: SedeEntry,
    automata: AhoCorasick,
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
        if p <= self.entry.ouro.unwrap_or(0) {
            Some(Color::Gold)
        } else if p <= self.entry.prata.unwrap_or(0) {
            Some(Color::Silver)
        } else if p <= self.entry.bronze.unwrap_or(0) {
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
            automata: AhoCorasick::new_auto_configured(&self.codes),
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
    pub fn into_contest(self) -> Contest {
        let entry_map: HashMap<String, SedeEntry> = self
            .sedes
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
    use super::*;

    #[test]
    fn test_config_patterns() {
        let sede = SedeEntry {
            name: "sede-name".into(),
            codes: ["teambr", "teammx"].into_iter().map(String::from).collect(),
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

        assert!(secret
            .get_sede_by_secret("key")
            .unwrap()
            .team_belongs_str("teambr$"),);
        assert!(secret
            .get_sede_by_secret("key")
            .unwrap()
            .team_belongs_str("teammx$"));
        assert!(secret
            .get_sede_by_secret("key")
            .unwrap()
            .team_belongs_str("$teammx$"));
        assert!(secret
            .get_sede_by_secret("key")
            .unwrap()
            .team_belongs_str("$teammx$"));
        assert!(secret
            .get_sede_by_secret("key")
            .unwrap()
            .team_belongs_str("$teammx"));
        assert!(secret
            .get_sede_by_secret("key")
            .unwrap()
            .team_belongs_str("$teammx"));

        assert!(!secret
            .get_sede_by_secret("key")
            .unwrap()
            .team_belongs_str("tea#mbr$"));
        assert!(!secret
            .get_sede_by_secret("key")
            .unwrap()
            .team_belongs_str("tea#mmx$"));
        assert!(!secret
            .get_sede_by_secret("key")
            .unwrap()
            .team_belongs_str("$te#ammx$"));
        assert!(!secret
            .get_sede_by_secret("key")
            .unwrap()
            .team_belongs_str("$te#ammx$"));
        assert!(!secret
            .get_sede_by_secret("key")
            .unwrap()
            .team_belongs_str("$te#ammx"));
        assert!(!secret
            .get_sede_by_secret("key")
            .unwrap()
            .team_belongs_str("$te#ammx"));

        assert!(!secret
            .get_sede_by_secret("key")
            .unwrap()
            .team_belongs_str("teamag"));
        assert!(!secret
            .get_sede_by_secret("key")
            .unwrap()
            .team_belongs_str("teamag$"));
        assert!(!secret
            .get_sede_by_secret("key")
            .unwrap()
            .team_belongs_str("$teamag$"));
    }
}
