pub enum Body {
    None,
    Empty,
    Bytes(&'static [u8]),
    Vector(Vec<u8>),
}

impl const From<&'static str> for Body {
    fn from(s: &'static str) -> Self {
        Self::Bytes(s.as_bytes())
    }
}

impl const From<&'static [u8]> for Body {
    fn from(s: &'static [u8]) -> Self {
        Self::Bytes(s)
    }
}

impl From<String> for Body {
    fn from(s: String) -> Self {
        Self::Vector(s.as_bytes().to_vec())
    }
}

impl From<Vec<u8>> for Body {
    fn from(s: Vec<u8>) -> Self {
        Self::Vector(s)
    }
}
