pub struct Sede {
    pub name: String,
    pub source: String,
    pub codes : Vec<String>,
}

impl Sede {
    pub fn new(name :&str, source:&str, code :&str) -> Self {
        Self::supersede(name, source, vec![code])
    }
    pub fn supersede(name:&str, source:&str, codes: Vec<&str>) -> Self {
        Self {
            name: name.to_string(),
            source: source.to_string(),
            codes : codes.iter().map(|c| c.to_string()).collect(),
        }
    }
}

pub struct Contest {
    pub host : String,
    pub sedes : Vec<Sede>,
}

impl Contest {
    pub fn new(host:&str, sedes:Vec<Sede>) -> Self {
        Self { host: host.to_string(), sedes }
    }
}

