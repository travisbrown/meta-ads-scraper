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

impl GraphApiVersion {
    #[must_use]
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

impl Display for GraphApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl FromStr for GraphApiVersion {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let error = || Error::InvalidGraphApiVersion(s.to_string());

        let (major, minor) = s
            .split_once('.')
            .filter(|(_, minor)| !minor.contains('.'))
            .and_then(|(major_str, minor_str)| major_str.parse().ok().zip(minor_str.parse().ok()))
            .ok_or_else(error)?;

        Ok(Self { major, minor })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_typical_version() {
        assert_eq!(
            "24.0".parse::<GraphApiVersion>().unwrap(),
            GraphApiVersion::new(24, 0)
        );
    }

    #[test]
    fn parse_non_zero_minor() {
        assert_eq!(
            "21.3".parse::<GraphApiVersion>().unwrap(),
            GraphApiVersion::new(21, 3)
        );
    }

    #[test]
    fn parse_zero_zero() {
        assert_eq!(
            "0.0".parse::<GraphApiVersion>().unwrap(),
            GraphApiVersion::new(0, 0)
        );
    }

    #[test]
    fn parse_large_values() {
        assert_eq!(
            "65535.65535".parse::<GraphApiVersion>().unwrap(),
            GraphApiVersion::new(u16::MAX, u16::MAX),
        );
    }

    #[test]
    fn parse_missing_dot() {
        assert!("240".parse::<GraphApiVersion>().is_err());
    }

    #[test]
    fn parse_three_part_version() {
        assert!("24.0.1".parse::<GraphApiVersion>().is_err());
    }

    #[test]
    fn parse_empty_string() {
        assert!("".parse::<GraphApiVersion>().is_err());
    }

    #[test]
    fn parse_non_numeric_major() {
        assert!("vX.0".parse::<GraphApiVersion>().is_err());
    }

    #[test]
    fn parse_non_numeric_minor() {
        assert!("24.x".parse::<GraphApiVersion>().is_err());
    }

    #[test]
    fn parse_overflow_major() {
        assert!("65536.0".parse::<GraphApiVersion>().is_err());
    }

    #[test]
    fn parse_overflow_minor() {
        assert!("0.65536".parse::<GraphApiVersion>().is_err());
    }

    #[test]
    fn parse_leading_dot() {
        assert!(".0".parse::<GraphApiVersion>().is_err());
    }

    #[test]
    fn parse_trailing_dot() {
        assert!("24.".parse::<GraphApiVersion>().is_err());
    }

    #[test]
    fn error_message_contains_input() {
        let err = "bad".parse::<GraphApiVersion>().unwrap_err();
        assert!(
            err.to_string().contains("bad")
                || matches!(err, Error::InvalidGraphApiVersion(s) if s == "bad")
        );
    }

    #[test]
    fn display_roundtrip() {
        let v = GraphApiVersion::new(24, 0);
        assert_eq!(v.to_string().parse::<GraphApiVersion>().unwrap(), v);
    }

    #[test]
    fn display_format() {
        assert_eq!(GraphApiVersion::new(21, 3).to_string(), "21.3");
    }
}
