use {
    crate::{error::InternalError, extractor::Extractor, Error, HttpRequest},
    std::ops::{Deref, DerefMut},
};

pub struct Header<const KEY: &'static str> {
    value: String,
}

impl<const KEY: &'static str> Deref for Header<KEY> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str> DerefMut for Header<KEY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<const KEY: &'static str> Extractor for Header<KEY> {
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        if let Some(value) = req
            .header_data
            .headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(KEY))
            .map(|(_, value)| value.to_string())
        {
            Ok(Self { value })
        } else {
            Err(InternalError::BadRequest(format!(
                "HTTP request did not contain the header `{}`",
                KEY
            )))
        }
    }
}

pub struct OptionalHeader<const KEY: &'static str> {
    value: Option<String>,
}

impl<const KEY: &'static str> Deref for OptionalHeader<KEY> {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str> DerefMut for OptionalHeader<KEY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<const KEY: &'static str> Extractor for OptionalHeader<KEY> {
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            value: req
                .header_data
                .headers
                .iter()
                .find(|(key, _)| key.eq_ignore_ascii_case(KEY))
                .map(|(_, value)| value.to_string()),
        })
    }
}
