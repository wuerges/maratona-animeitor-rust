use std::collections::HashMap;

use data::configdata::Sede;

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
