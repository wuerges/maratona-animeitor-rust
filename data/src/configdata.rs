use std::collections::HashMap;

use aho_corasick::AhoCorasick;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Sede {
    pub name: String,
    pub codes: Vec<String>,
    pub style: Option<String>,
    pub premiacao: bool,
    pub ouro: Option<usize>,
    pub prata: Option<usize>,
    pub bronze: Option<usize>,
    pub contest: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Escola {
    pub name: String,
    pub code: String,
    pub logo: String,
}

// [[teams]]
// login="teambrsp066"
// nome="Nome do Config"
// foto="KLM.png"
// musica="https://youtu.be/gdG4xbU8cZo"
// comentario="Na foto: Prof. Acauan (Coach), Markus Kaul, Leandro Schillreff, Miller Raycell"

#[derive(Deserialize, Serialize, Debug)]
pub struct TeamEntry {
    pub login: String,
    pub nome: Option<String>,
    pub foto: Option<String>,
    pub musica: Option<String>,
    pub comentario: Option<String>,
}

impl Sede {
    pub fn check_filter_login(url_filter: &Option<Vec<String>>, t: &str) -> bool {
        match url_filter {
            None => true,
            Some(tot) => {
                for f in tot {
                    if t.contains(f) {
                        return true;
                    }
                }
                false
            }
        }
    }

    pub fn check_login(&self, t: &str) -> bool {
        for f in &self.codes {
            if t.contains(f) {
                return true;
            }
        }
        false
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigContest {
    pub sedes: Vec<Sede>,
    pub escolas: Vec<Escola>,
    pub teams: Vec<TeamEntry>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigSedes {
    pub sedes: Vec<Sede>,
}

#[derive(Debug, Clone)]
pub struct ConfigSecretPatterns {
    pub secrets: Box<HashMap<String, AhoCorasick>>,
}

impl ConfigSecretPatterns {
    fn new(patterns: HashMap<String, Vec<String>>) -> Self {
        Self {
            secrets: Box::new(
                patterns
                    .into_iter()
                    .map(|(key, teams)| (key, AhoCorasick::new(teams)))
                    .collect(),
            ),
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
    pub fn get_patterns(self, sedes: &ConfigSedes) -> ConfigSecretPatterns {
        let salt = self.salt.unwrap_or_default();
        ConfigSecretPatterns::new(
            self.secrets
                .into_iter()
                .filter_map(|sede_secret| {
                    let complete = format!("{}{}", salt, &sede_secret.secret);
                    sedes
                        .sedes
                        .iter()
                        .find_map(|sede| {
                            (sede.name == sede_secret.name).then_some(sede.codes.clone())
                        })
                        .map(|codes| (complete, codes))
                })
                .collect(),
        )
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigEscolas {
    pub escolas: Vec<Escola>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigTeams {
    pub teams: Vec<TeamEntry>,
}

impl ConfigContest {
    pub fn dummy() -> Self {
        Self {
            sedes: Vec::new(),
            escolas: Vec::new(),
            teams: Vec::new(),
        }
    }

    pub fn from_config(sedes: Vec<Sede>, escolas: Vec<Escola>, teams: Vec<TeamEntry>) -> Self {
        Self {
            sedes,
            escolas,
            teams,
        }
    }

    pub fn new(sedes: Vec<Sede>) -> Self {
        Self {
            sedes,
            escolas: Vec::new(),
            teams: Vec::new(),
        }
    }

    pub fn get_sede_team(&self, team: &str) -> Option<&Sede> {
        self.sedes.iter().find(|&sede| sede.check_login(team))
    }

    pub fn get_sede_nome_sede(&self, name: &str) -> Option<&Sede> {
        self.sedes.iter().find(|&sede| sede.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_patterns() {
        let config = ConfigSecretPatterns::new(HashMap::from([(
            "key".into(),
            ["teambr", "teammx"].into_iter().map(String::from).collect(),
        )]));

        assert!(config.secrets.get("key").unwrap().is_match("teambr$"));
        assert!(config.secrets.get("key").unwrap().is_match("teammx$"));
        assert!(config.secrets.get("key").unwrap().is_match("$teammx$"));
        assert!(config.secrets.get("key").unwrap().is_match("$teammx$"));
        assert!(config.secrets.get("key").unwrap().is_match("$teammx"));
        assert!(config.secrets.get("key").unwrap().is_match("$teammx"));

        assert!(!config.secrets.get("key").unwrap().is_match("tea#mbr$"));
        assert!(!config.secrets.get("key").unwrap().is_match("tea#mmx$"));
        assert!(!config.secrets.get("key").unwrap().is_match("$te#ammx$"));
        assert!(!config.secrets.get("key").unwrap().is_match("$te#ammx$"));
        assert!(!config.secrets.get("key").unwrap().is_match("$te#ammx"));
        assert!(!config.secrets.get("key").unwrap().is_match("$te#ammx"));

        assert!(!config.secrets.get("key").unwrap().is_match("teamag"));
        assert!(!config.secrets.get("key").unwrap().is_match("teamag$"));
        assert!(!config.secrets.get("key").unwrap().is_match("$teamag$"));
    }
}
