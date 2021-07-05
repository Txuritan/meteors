mod body;
mod method;
pub(crate) mod request;
pub(crate) mod response;
mod status;
mod version;

pub(crate) use self::body::Body;

pub use self::{method::Method, status::StatusCode, version::Version};

#[derive(Debug)]
pub enum Error {
    InvalidRequest,

    ParseMissingMeta,
    ParseMetaMissingMethod,
    ParseMetaMissingUrl,
    ParseMetaMissingVersion,

    ParseUnknownMethod,
    ParseUnknownVersion,
}
