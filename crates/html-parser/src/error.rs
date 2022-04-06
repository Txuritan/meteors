#[derive(Debug)]
pub enum Error {
    Parsing(String),
    IllegalInclude(String),
    InvalidRoot,
    AttributeCreation(String),
    ElementCreation(Option<String>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parsing(err) => err.fmt(f),
            Error::IllegalInclude(include) => {
                write!(f, "A document fragment should not include {}", include)
            }
            Error::InvalidRoot => write!(f, "A document can only have html as root"),
            Error::AttributeCreation(at) => write!(f, "Failed to create attribute at rule: {}", at),
            Error::ElementCreation(at) => match at {
                Some(at) => write!(f, "Failed to create element at rule: {}", at),
                None => write!(f, "Failed to create element"),
            },
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub type Result<T> = std::result::Result<T, Error>;
