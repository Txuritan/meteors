use crate::{extractor::ExtractorError, http::HttpError};

pub enum Error {
    Extractor(ExtractorError),
    Http(HttpError),
}

impl From<ExtractorError> for Error {
    fn from(err: ExtractorError) -> Self {
        Error::Extractor(err)
    }
}

impl From<HttpError> for Error {
    fn from(err: HttpError) -> Self {
        Error::Http(err)
    }
}
