use service::volume::Volume;

use crate::pair_arg::PairArg;

pub mod config_secret;
pub mod pair_arg;
pub mod sentry;
pub mod test_revelation;

impl From<PairArg> for Volume {
    fn from(PairArg { first, second }: PairArg) -> Self {
        Self {
            folder: first,
            path: second,
        }
    }
}
