mod body;
pub mod headers;
mod method;
pub(crate) mod request;
pub(crate) mod response;
mod status;
mod version;

pub(crate) use self::body::Body;

pub use self::{method::Method, status::StatusCode, version::Version};

#[cfg(feature = "fuzzing")]
pub use self::request::HttpRequest;

#[derive(Debug)]
pub enum HttpError {
    InvalidRequest,

    ParseMissingMeta,
    ParseMetaMissingMethod,
    ParseMetaMissingUrl,
    ParseMetaMissingVersion,

    ParseUnknownMethod,
    ParseUnknownVersion,

    Io(std::io::Error),
    ParseInt(std::num::ParseIntError),
}

impl const From<std::io::Error> for HttpError {
    fn from(v: std::io::Error) -> Self {
        Self::Io(v)
    }
}

impl const From<std::num::ParseIntError> for HttpError {
    fn from(v: std::num::ParseIntError) -> Self {
        Self::ParseInt(v)
    }
}
