use std::{
    collections::HashMap,
    fmt::{Debug, Display},
};

use derivative::Derivative;
use regex::RegexSet;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Derivative)]
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

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
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

impl Sede {
    pub fn team_belongs_str(&self, team_login: &str) -> bool {
        self.automata.is_match(team_login)
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

#[derive(Serialize, Deserialize, Debug, Clone)]
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
