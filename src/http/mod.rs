pub mod body;
pub mod method;
pub mod request;
pub mod response;
pub mod status;
pub mod version;

pub use self::{
    body::Body, method::Method, request::HttpRequest, response::HttpResponse, status::StatusCode,
    version::Version,
};

#[derive(Debug)]
pub enum HttpError {
    InvalidRequest,

    ParseMissingMeta,
    ParseMetaMissingMethod,
    ParseMetaMissingUrl,
    ParseMetaMissingVersion,

    ParseUnknownMethod,
    ParseUnknownVersion,
}
