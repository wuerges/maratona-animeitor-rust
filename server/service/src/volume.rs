use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Volume {
    /// Local folder to host.
    pub folder: String,
    /// Relative path from host /.
    pub path: String,
}

impl FromStr for Volume {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (folder, path) = s.trim().split_once(':').ok_or("delimiter `:' not found.")?;

        Ok(Volume {
            folder: folder.to_string(),
            path: path.to_string(),
        })
    }
}
