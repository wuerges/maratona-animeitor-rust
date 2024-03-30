use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct PairArg {
    pub first: String,
    pub second: String,
}

impl FromStr for PairArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (first, second) = s.trim().split_once(':').ok_or("delimiter `:' not found.")?;

        Ok(PairArg {
            first: first.to_string(),
            second: second.to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct FromPairArg<T: From<PairArg>>(T);

impl<T: From<PairArg>> FromPairArg<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> FromStr for FromPairArg<T>
where
    T: From<PairArg>,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse::<PairArg>()?.into()))
    }
}
