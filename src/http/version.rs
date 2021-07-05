use {
    crate::http::Error,
    std::{
        fmt::{self, Display, Formatter},
        str::FromStr,
    },
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Version {
    Http09,
    Http10,
    Http11,
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Version::Http09 => write!(f, "HTTP/0.9"),
            Version::Http10 => write!(f, "HTTP/1.0"),
            Version::Http11 => write!(f, "HTTP/1.1"),
        }
    }
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/0.9" => Ok(Self::Http09),
            "HTTP/1.0" => Ok(Self::Http10),
            "HTTP/1.1" => Ok(Self::Http11),
            _ => Err(Error::ParseUnknownVersion),
        }
    }
}
