use {
    crate::{error::InternalError, extractor::Extractor, Error, HttpRequest},
    std::{
        fmt::Debug,
        ops::{Deref, DerefMut},
        str::FromStr,
    },
};

pub struct Param<const KEY: &'static str> {
    value: String,
}

impl<const KEY: &'static str> const Deref for Param<KEY> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str> const DerefMut for Param<KEY> {
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

impl<const KEY: &'static str> const Deref for OptionalParam<KEY> {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str> const DerefMut for OptionalParam<KEY> {
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

pub struct ParseParam<const KEY: &'static str, T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    value: T,
}

impl<const KEY: &'static str, T> const Deref for ParseParam<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str, T> const DerefMut for ParseParam<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<const KEY: &'static str, T> Extractor for ParseParam<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        if let Some(value) = req.params.get(KEY).map(|s| T::from_str(s)) {
            match value {
                Ok(value) => Ok(Self { value }),
                Err(err) => Err(InternalError::BadRequest(format!(
                    "HTTP request URL parameter with key `{}` could not be parsed: {:?}",
                    KEY, err
                ))),
            }
        } else {
            Err(InternalError::BadRequest(format!(
                "HTTP request URL parameters did not contain a value with the key `{}`",
                KEY
            )))
        }
    }
}
