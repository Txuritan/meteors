use {
    crate::{
        extractor::{Extractor, ExtractorError},
        HttpRequest,
    },
    std::ops::Deref,
};

pub struct Param<const KEY: &'static str> {
    value: String,
}

impl<const KEY: &'static str> Deref for Param<KEY> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str> Extractor for Param<KEY> {
    type Error = ExtractorError;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        if let Some(value) = req.params.get(KEY).cloned() {
            Ok(Self { value })
        } else {
            Err(ExtractorError::Missing)
        }
    }
}
