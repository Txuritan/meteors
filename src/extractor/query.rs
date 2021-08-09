use {
    crate::{error::InternalError, extractor::Extractor, Error, HttpRequest},
    std::{
        fmt::Debug,
        ops::{Deref, DerefMut},
        str::FromStr,
    },
};

fn get_value<'req>(req: &'req HttpRequest, key: &'static str) -> Option<&'req String> {
    req.header_data.query_params.get(key)
}

fn get_value_err<'req>(req: &'req HttpRequest, key: &'static str) -> Result<&'req String, Error> {
    match get_value(req, key) {
        Some(v) => Ok(v),
        None => Err(InternalError::BadRequest(format!(
            "HTTP request URL query did not contain a value with the key `{}`",
            key
        ))),
    }
}

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
        match get_value_err(&*req, KEY) {
            Ok(value) => Ok(Self {
                value: value.clone(),
            }),
            Err(err) => Err(err),
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
            value: get_value(&*req, KEY).cloned(),
        })
    }
}

pub struct ParseQuery<const KEY: &'static str, T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    value: T,
}

impl<const KEY: &'static str, T> const Deref for ParseQuery<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str, T> const DerefMut for ParseQuery<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<const KEY: &'static str, T> Extractor for ParseQuery<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        match get_value_err(&*req, KEY) {
            Ok(value) => match T::from_str(value) {
                Ok(value) => Ok(Self { value }),
                Err(err) => Err(InternalError::BadRequest(format!(
                    "HTTP request URL query with key `{}` could not be parsed: {:?}",
                    KEY, err
                ))),
            },
            Err(err) => Err(err),
        }
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
