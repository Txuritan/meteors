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
#[allow(clippy::enum_variant_names)] // for the `Parse` prefix, in case i need to add more variants
pub enum HttpError {
    ParseMissingMeta,
    ParseMetaMissingMethod,
    ParseMetaMissingUrl,
    ParseMetaMissingVersion,

    ParseUnknownMethod,
    ParseUnknownVersion,
}
