use crate::pair_arg::PairArg;

#[derive(Debug, Clone)]
pub struct Volume {
    /// Local folder to host.
    pub folder: String,
    /// Relative path from host /.
    pub path: String,
}

impl From<PairArg> for Volume {
    fn from(PairArg { first, second }: PairArg) -> Self {
        Self {
            folder: first,
            path: second,
        }
    }
}
