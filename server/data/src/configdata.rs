use std::collections::HashMap;

use aho_corasick::AhoCorasick;
use serde::{Deserialize, Serialize};

use crate::Team;

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct SedeEntry {
    pub name: String,
    pub codes: Vec<String>,
    pub style: Option<String>,
    pub ouro: Option<usize>,
    pub prata: Option<usize>,
    pub bronze: Option<usize>,
    pub contest: Option<String>,
}

#[derive(Debug)]
pub struct Sede {
    pub name: String,
    pub style: Option<String>,
    pub ouro: Option<usize>,
    pub prata: Option<usize>,
    pub bronze: Option<usize>,
    pub contest: Option<String>,
    automata: AhoCorasick,
}

impl Sede {
    pub fn team_belongs(&self, team: &Team) -> bool {
        self.team_belongs_str(&team.login)
    }

    pub fn team_belongs_str(&self, team_login: &str) -> bool {
        self.automata.is_match(team_login)
    }

    pub fn premio(&self, p: usize) -> &str {
        if p <= self.ouro.unwrap_or(0) {
            "ouro"
        } else if p <= self.prata.unwrap_or(0) {
            "prata"
        } else if p <= self.bronze.unwrap_or(0) {
            "bronze"
        } else {
            "semcor"
        }
    }
}

impl SedeEntry {
    pub fn build(&self) -> Sede {
        Sede {
            name: self.name.clone(),
            style: self.style.clone(),
            automata: AhoCorasick::new_auto_configured(&self.codes),
            ouro: self.ouro.clone(),
            prata: self.prata.clone(),
            bronze: self.bronze.clone(),
            contest: self.contest.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigContest {
    pub sedes: Vec<SedeEntry>,
}

#[derive(Debug)]
pub struct ConfigSecretPatterns {
    pub secrets: Box<HashMap<String, Sede>>,
    pub parameters: Box<HashMap<String, SedeEntry>>,
}

impl ConfigSecretPatterns {
    fn new(patterns: HashMap<String, SedeEntry>) -> Self {
        Self {
            secrets: Box::new(
                patterns
                    .iter()
                    .map(|(key, sede)| (key.clone(), sede.build()))
                    .collect(),
            ),
            parameters: Box::new(patterns),
        }
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

impl ConfigSecret {
    pub fn get_patterns(self, sedes: &ConfigContest) -> ConfigSecretPatterns {
        let salt = self.salt.unwrap_or_default();
        ConfigSecretPatterns::new(
            self.secrets
                .into_iter()
                .filter_map(|sede_secret| {
                    let complete = format!("{}{}", salt, &sede_secret.secret);
                    sedes
                        .sedes
                        .iter()
                        .find_map(|sede| (sede.name == sede_secret.name).then_some(sede))
                        .map(|sede| (complete, sede.clone()))
                })
                .collect(),
        )
    }
}

impl ConfigContest {
    pub fn dummy() -> Self {
        Self { sedes: Vec::new() }
    }

    pub fn get_sede_nome_sede(&self, name: &str) -> Option<Sede> {
        self.sedes
            .iter()
            .find(|&sede| sede.name == name)
            .map(|s| s.build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_patterns() {
        let mut sede = SedeEntry::default();

        sede.codes = ["teambr", "teammx"].into_iter().map(String::from).collect();

        let config = ConfigSecretPatterns::new(HashMap::from([("key".into(), sede)]));

        assert!(config
            .secrets
            .get("key")
            .unwrap()
            .team_belongs_str("teambr$"));
        assert!(config
            .secrets
            .get("key")
            .unwrap()
            .team_belongs_str("teammx$"));
        assert!(config
            .secrets
            .get("key")
            .unwrap()
            .team_belongs_str("$teammx$"));
        assert!(config
            .secrets
            .get("key")
            .unwrap()
            .team_belongs_str("$teammx$"));
        assert!(config
            .secrets
            .get("key")
            .unwrap()
            .team_belongs_str("$teammx"));
        assert!(config
            .secrets
            .get("key")
            .unwrap()
            .team_belongs_str("$teammx"));

        assert!(!config
            .secrets
            .get("key")
            .unwrap()
            .team_belongs_str("tea#mbr$"));
        assert!(!config
            .secrets
            .get("key")
            .unwrap()
            .team_belongs_str("tea#mmx$"));
        assert!(!config
            .secrets
            .get("key")
            .unwrap()
            .team_belongs_str("$te#ammx$"));
        assert!(!config
            .secrets
            .get("key")
            .unwrap()
            .team_belongs_str("$te#ammx$"));
        assert!(!config
            .secrets
            .get("key")
            .unwrap()
            .team_belongs_str("$te#ammx"));
        assert!(!config
            .secrets
            .get("key")
            .unwrap()
            .team_belongs_str("$te#ammx"));

        assert!(!config
            .secrets
            .get("key")
            .unwrap()
            .team_belongs_str("teamag"));
        assert!(!config
            .secrets
            .get("key")
            .unwrap()
            .team_belongs_str("teamag$"));
        assert!(!config
            .secrets
            .get("key")
            .unwrap()
            .team_belongs_str("$teamag$"));
    }
}
