use crate::{extractor::ExtractorError, http};

pub enum Error {
    Extractor(ExtractorError),
    Http(http::Error),
}

impl From<ExtractorError> for Error {
    fn from(err: ExtractorError) -> Self {
        Error::Extractor(err)
    }
}

impl From<http::Error> for Error {
    fn from(err: http::Error) -> Self {
        Error::Http(err)
    }
}
