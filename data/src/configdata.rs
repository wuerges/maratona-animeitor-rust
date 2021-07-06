use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Sede {
    pub name: String,
    pub codes: Vec<String>,
    pub premiacao: bool,
}

#[derive(Deserialize, Serialize)]
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

#[derive(Deserialize, Serialize)]
pub struct TeamEntry {
    pub login: String,
    pub nome: Option<String>,
    pub foto: Option<String>,
    pub musica: Option<String>,
    pub comentario: Option<String>
}

impl Sede {

    pub fn check_filter_login(url_filter: &Option<Vec<String>>, t: &String) -> bool {
        match url_filter {
            None => true,
            Some(tot) => {
                for f in tot {
                    if t.find(f).is_some() {
                        return true;
                    }
                }
                return false;
            }
        }
    }

    pub fn check_login(&self, t: &String) -> bool {
        for f in &self.codes {
            if t.find(f).is_some() {
                return true;
            }
        }
        return false;
    }
}

#[derive(Serialize, Deserialize)]
pub struct ConfigContest {
    pub sedes: Vec<Sede>,
    pub escolas: Vec<Escola>,
    pub teams: Vec<TeamEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigSedes {
    pub sedes: Vec<Sede>,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigEscolas {
    pub escolas: Vec<Escola>,
}

#[derive(Serialize, Deserialize)]
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

    pub fn from_config(sedes: Vec<Sede>, escolas: Vec<Escola>, teams: Vec<TeamEntry> ) -> Self {
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

    pub fn get_sede(&self, team: &String) -> Option<&Sede> {
        for sede in &self.sedes {
            if sede.check_login(team) {
                return Some(&sede);
            }
        }
        None
    }
}
