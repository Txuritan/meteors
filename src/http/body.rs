pub enum Body {
    None,
    Empty,
    Bytes(Vec<u8>),
}

impl From<&'static str> for Body {
    fn from(s: &'static str) -> Self {
        Self::Bytes(s.as_bytes().to_vec())
    }
}

impl From<&'static [u8]> for Body {
    fn from(s: &'static [u8]) -> Self {
        Self::Bytes(s.to_vec())
    }
}

impl From<String> for Body {
    fn from(s: String) -> Self {
        Self::Bytes(s.as_bytes().to_vec())
    }
}

impl From<Vec<u8>> for Body {
    fn from(s: Vec<u8>) -> Self {
        Self::Bytes(s)
    }
}
