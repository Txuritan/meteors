use {
    crate::{
        extractor::{Extractor, ExtractorError},
        HttpRequest,
    },
    std::ops::Deref,
};

pub struct Query<const KEY: &'static str> {
    value: String,
}

impl<const KEY: &'static str> Deref for Query<KEY> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str> Extractor for Query<KEY> {
    type Error = ExtractorError;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        if let Some(value) = req.header_data.query_params.get(KEY).cloned() {
            Ok(Self { value })
        } else {
            Err(ExtractorError::Missing)
        }
    }
}

pub struct OptionalQuery<const KEY: &'static str> {
    value: Option<String>,
}

impl<const KEY: &'static str> Deref for OptionalQuery<KEY> {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str> Extractor for OptionalQuery<KEY> {
    type Error = ExtractorError;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            value: req.header_data.query_params.get(KEY).cloned(),
        })
    }
}

pub struct RawQuery {
    value: String,
}

impl Deref for RawQuery {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl Extractor for RawQuery {
    type Error = ExtractorError;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            value: req.header_data.query.clone(),
        })
    }
}
