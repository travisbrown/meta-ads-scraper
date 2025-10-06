use std::{fmt::Display, str::FromStr};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid Graph API version")]
    InvalidGraphApiVersion(String),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct GraphApiVersion {
    pub major: u16,
    pub minor: u16,
}

impl Display for GraphApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl FromStr for GraphApiVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('.').collect::<Vec<_>>();

        if parts.len() == 2 {
            let (major, minor) = parts[0]
                .parse()
                .ok()
                .zip(parts[1].parse().ok())
                .ok_or_else(|| Error::InvalidGraphApiVersion(s.to_string()))?;

            Ok(Self { major, minor })
        } else {
            Err(Error::InvalidGraphApiVersion(s.to_string()))
        }
    }
}
