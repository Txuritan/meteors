use {super::HttpError, std::str::FromStr};

pub enum Version {
    Http09,
    Http10,
    Http11,
}

impl FromStr for Version {
    type Err = HttpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/0.9" => Ok(Self::Http09),
            "HTTP/1.0" => Ok(Self::Http10),
            "HTTP/1.1" => Ok(Self::Http11),
            _ => Err(HttpError::ParseUnknownVersion),
        }
    }
}
