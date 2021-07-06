use {
    crate::{error::InternalError, extractor::Extractor, Error, HttpRequest},
    std::ops::{Deref, DerefMut},
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

impl<const KEY: &'static str> DerefMut for Param<KEY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<const KEY: &'static str> Extractor for Param<KEY> {
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        if let Some(value) = req.params.get(KEY).cloned() {
            Ok(Self { value })
        } else {
            Err(InternalError::BadRequest(format!(
                "HTTP request URL parameters did not contain a value with the key `{}`",
                KEY
            )))
        }
    }
}

pub struct OptionalParam<const KEY: &'static str> {
    value: Option<String>,
}

impl<const KEY: &'static str> Deref for OptionalParam<KEY> {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str> DerefMut for OptionalParam<KEY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<const KEY: &'static str> Extractor for OptionalParam<KEY> {
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            value: req.params.get(KEY).cloned(),
        })
    }
}
