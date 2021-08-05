use {
    crate::{error::InternalError, extractor::Extractor, Error, HttpRequest},
    std::ops::{Deref, DerefMut},
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

impl<const KEY: &'static str> DerefMut for Query<KEY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<const KEY: &'static str> Extractor for Query<KEY> {
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        if let Some(value) = req.header_data.query_params.get(KEY).cloned() {
            Ok(Self { value })
        } else {
            Err(InternalError::BadRequest(format!(
                "HTTP request URL query did not contain a value with the key `{}`",
                KEY
            )))
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

impl<const KEY: &'static str> DerefMut for OptionalQuery<KEY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<const KEY: &'static str> Extractor for OptionalQuery<KEY> {
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            value: req.header_data.query_params.get(KEY).cloned(),
        })
    }
}

pub struct RawQuery {
    value: String,
}

impl const Deref for RawQuery {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl const DerefMut for RawQuery {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl Extractor for RawQuery {
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            value: req.header_data.query.clone(),
        })
    }
}
