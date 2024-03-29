use std::collections::HashMap;

use aho_corasick::AhoCorasick;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::Team;

#[derive(Deserialize, Serialize, Debug, Clone, Default, ToSchema)]
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
    /// Contest that owns this site.
    pub contest: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Sede {
    pub entry: SedeEntry,
    contest: Option<Box<Sede>>,
    automata: AhoCorasick,
}

impl Sede {
    pub fn contest(&self) -> Option<&Sede> {
        self.contest.as_deref()
    }

    pub fn team_belongs(&self, team: &Team) -> bool {
        self.team_belongs_str(&team.login)
    }

    pub fn team_belongs_str(&self, team_login: &str) -> bool {
        self.automata.is_match(team_login)
    }

    pub fn premio(&self, p: usize) -> &str {
        if p <= self.entry.ouro.unwrap_or(0) {
            "ouro"
        } else if p <= self.entry.prata.unwrap_or(0) {
            "prata"
        } else if p <= self.entry.bronze.unwrap_or(0) {
            "bronze"
        } else {
            "semcor"
        }
    }
}

impl SedeEntry {
    pub fn into_sede(&self, all_sedes: &HashMap<String, SedeEntry>) -> Sede {
        Sede {
            entry: self.clone(),
            automata: AhoCorasick::new_auto_configured(&self.codes),
            contest: self
                .contest
                .as_ref()
                .and_then(|c| all_sedes.get(c))
                .map(|s| Box::new(s.into_sede(&HashMap::new()))),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
/// Site configuration for contest.
pub struct ConfigContest {
    /// A site entry.
    pub sedes: Vec<SedeEntry>,
}

impl ConfigContest {
    pub fn into_contest(self) -> Contest {
        let entry_map: HashMap<String, SedeEntry> = self
            .sedes
            .into_iter()
            .map(|sede| (sede.name.clone(), sede))
            .collect();

        Contest {
            sedes: entry_map
                .iter()
                .map(|(name, entry)| (name.clone(), entry.into_sede(&entry_map)))
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct Contest {
    pub sedes: HashMap<String, Sede>,
}

impl Contest {
    pub fn get_sede_nome_sede(&self, name: &str) -> Option<&Sede> {
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
    pub salt: Option<String>,
    pub secrets: Vec<SedeSecret>,
}

#[derive(Debug)]
pub struct Secret {
    pub sedes_by_secret: HashMap<String, Sede>,
}

impl Secret {
    pub fn get_sede_by_secret(&self, key: &str) -> Option<&Sede> {
        self.sedes_by_secret.get(key)
    }
}

impl ConfigSecret {
    pub fn into_secret(self, sedes: &Contest) -> Secret {
        let salt = self.salt.unwrap_or_default();
        let sedes_by_secret = self
            .secrets
            .into_iter()
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

impl ConfigContest {
    pub fn dummy() -> Self {
        Self { sedes: Vec::new() }
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

        let config_contest = ConfigContest { sedes: vec![sede] };
        let contest = config_contest.into_contest();

        let config_secret = ConfigSecret {
            salt: None,
            secrets: vec![SedeSecret {
                name: "sede-name".into(),
                secret: "key".into(),
            }],
        };
        let secret = config_secret.into_secret(&contest);

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
