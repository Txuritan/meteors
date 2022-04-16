use std::{
    ops::{Deref, DerefMut},
    str::FromStr,
};

use crate::{error::InternalError, extractor::Extractor, http::HttpRequest, Error};

fn get_value<'req>(req: &'req HttpRequest, key: &'static str) -> Option<&'req String> {
    req.headers
        .iter()
        .find(|(k, _)| k.0.eq_ignore_ascii_case(key))
        .map(|(_, value)| value)
}

fn get_value_err<'req>(req: &'req HttpRequest, key: &'static str) -> Result<&'req String, Error> {
    match get_value(req, key) {
        Some(v) => Ok(v),
        None => Err(InternalError::BadRequest(crate::wrapper::format!(
            "HTTP request did not contain the header `{}`",
            key
        ))),
    }
}

pub struct Header<const KEY: &'static str> {
    value: String,
}

impl<const KEY: &'static str> const Deref for Header<KEY> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str> const DerefMut for Header<KEY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<const KEY: &'static str> Extractor for Header<KEY> {
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

pub struct OptionalHeader<const KEY: &'static str> {
    value: Option<String>,
}

impl<const KEY: &'static str> const Deref for OptionalHeader<KEY> {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str> const DerefMut for OptionalHeader<KEY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<const KEY: &'static str> Extractor for OptionalHeader<KEY> {
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            value: get_value(&*req, KEY).cloned(),
        })
    }
}

pub struct ParseHeader<const KEY: &'static str, T>
where
    T: FromStr,
    <T as FromStr>::Err: crate::wrapper::Debug,
{
    value: T,
}

impl<const KEY: &'static str, T> const Deref for ParseHeader<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: crate::wrapper::Debug,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str, T> const DerefMut for ParseHeader<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: crate::wrapper::Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<const KEY: &'static str, T> Extractor for ParseHeader<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: crate::wrapper::Debug,
{
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        match get_value_err(&*req, KEY) {
            Ok(value) => match T::from_str(value) {
                Ok(value) => Ok(Self { value }),
                Err(err) => Err(InternalError::BadRequest(crate::wrapper::format!(
                    "HTTP request header with key `{}` could not be parsed: {:?}",
                    KEY,
                    err
                ))),
            },
            Err(err) => Err(err),
        }
    }
}
