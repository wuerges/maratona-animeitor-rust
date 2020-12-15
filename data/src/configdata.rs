use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Sede {
    pub name: String,
    pub codes: Vec<String>,
    pub premiacao: bool,
    pub vagas: usize,
}

#[derive(Deserialize, Serialize)]
pub struct Escola {
    pub name: String,
    pub code: String,
    pub logo: String,
}

impl Sede {
    pub fn new(name: &str, code: &str, premiacao: bool, vagas: usize) -> Self {
        Self::supersede(name, vec![code], premiacao, vagas)
    }
    pub fn supersede(name: &str, codes: Vec<&str>, premiacao: bool, vagas: usize) -> Self {
        Self {
            name: name.to_string(),
            codes: codes.iter().map(|c| c.to_string()).collect(),
            premiacao,
            vagas,
        }
    }

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
}

impl ConfigContest {
    pub fn dummy() -> Self {
        Self {
            sedes: Vec::new(),
            escolas: Vec::new(),
        }
    }

    pub fn new(sedes: Vec<Sede>) -> Self {
        Self {
            sedes,
            escolas: Vec::new(),
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
