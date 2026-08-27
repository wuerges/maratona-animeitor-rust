#[derive(Debug, Clone)]
pub struct Volume {
    /// Local folder to host.
    pub folder: String,
    /// Relative path from host /.
    pub path: String,
}
