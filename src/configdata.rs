pub struct Sede {
    pub name: String,
    pub parent: String,
    pub codes : Vec<String>,
}

impl Sede {
    pub fn new(name :&str, parent:&str, code :&str) -> Self {
        Self::supersede(name, parent, vec![code])
    }
    pub fn supersede(name:&str, parent:&str, codes: Vec<&str>) -> Self {
        Self {
            name: name.to_string(),
            parent: parent.to_string(),
            codes : codes.iter().map(|c| c.to_string()).collect(),
        }
    }
}

pub struct Contest {
    pub host : String,
    pub salt : String,
    pub sedes : Vec<Sede>,
}

impl Contest {
    pub fn new(host:&str, salt:&str, sedes:Vec<Sede>) -> Self {
        Self { host: host.to_string(), salt: salt.to_string(), sedes }
    }
}

