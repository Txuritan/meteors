use std::{
    ops::{Deref, DerefMut},
    str::FromStr,
};

use crate::{
    extractor::Extractor,
    http::{HttpRequest, HttpResponse},
    response::IntoResponse,
};

fn get_value<'req>(req: &'req HttpRequest, key: &'static str) -> Option<&'req String> {
    req.headers
        .iter()
        .find(|(k, _)| k.0.eq_ignore_ascii_case(key))
        .map(|(_, value)| value)
}

fn get_value_err<'req, const KEY: &'static str>(
    req: &'req HttpRequest,
) -> Result<&'req String, HeaderMissingRejection<KEY>> {
    match get_value(req, KEY) {
        Some(v) => Ok(v),
        None => Err(HeaderMissingRejection {}),
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
    type Error = HeaderMissingRejection<KEY>;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        match get_value_err::<KEY>(&*req) {
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
    type Error = HeaderMissingRejection<KEY>;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            value: get_value(&*req, KEY).cloned(),
        })
    }
}

pub struct HeaderMissingRejection<const KEY: &'static str> {}

#[cfg(feature = "std")]
impl<const KEY: &'static str> IntoResponse for HeaderMissingRejection<KEY> {
    fn into_response(self) -> HttpResponse {
        std::format!("HTTP request header with key `{}` could not be found", KEY,).into_response()
    }
}

#[cfg(feature = "vfmt")]
impl<const KEY: &'static str> IntoResponse for HeaderMissingRejection<KEY> {
    fn into_response(self) -> HttpResponse {
        vfmt::format!("HTTP request header with key `{}` could not be found", KEY,).into_response()
    }
}

#[cfg(feature = "std")]
pub struct ParseHeader<const KEY: &'static str, T>(pub T)
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug;

#[cfg(feature = "std")]
impl<const KEY: &'static str, T> Extractor for ParseHeader<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        match get_value_err(&*req, KEY) {
            Ok(value) => match T::from_str(value) {
                Ok(value) => Ok(Self(value)),
                Err(err) => Err(InternalError::BadRequest(std::format!(
                    "HTTP request header with key `{}` could not be parsed: {:?}",
                    KEY,
                    err
                ))),
            },
            Err(err) => Err(err),
        }
    }
}

#[cfg(feature = "vfmt")]
pub struct ParseHeader<const KEY: &'static str, T>(pub T)
where
    T: FromStr,
    <T as FromStr>::Err: vfmt::uDebug;

#[cfg(feature = "vfmt")]
impl<const KEY: &'static str, T> Extractor for ParseHeader<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: vfmt::uDebug,
{
    type Error = HttpResponse;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        match get_value_err::<KEY>(&*req) {
            Ok(value) => match T::from_str(value) {
                Ok(value) => Ok(Self(value)),
                Err(err) => Err(HeaderRejection::<KEY, T> { err }.into_response()),
            },
            Err(err) => Err(err.into_response()),
        }
    }
}

pub struct HeaderRejection<const KEY: &'static str, T: FromStr> {
    err: <T as FromStr>::Err,
}

#[cfg(feature = "std")]
impl<const KEY: &'static str, T> IntoResponse for HeaderRejection<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    fn into_response(self) -> HttpResponse {
        std::format!(
            "HTTP request header with key `{}` could not be parsed: {:?}",
            KEY,
            self.err
        )
        .into_response()
    }
}

#[cfg(feature = "vfmt")]
impl<const KEY: &'static str, T> IntoResponse for HeaderRejection<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: vfmt::uDebug,
{
    fn into_response(self) -> HttpResponse {
        vfmt::format!(
            "HTTP request header with key `{}` could not be parsed: {:?}",
            KEY,
            self.err
        )
        .into_response()
    }
}
