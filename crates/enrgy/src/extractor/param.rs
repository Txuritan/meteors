use std::{
    ops::{Deref, DerefMut},
    str::FromStr,
};

use crate::{
    extractor::Extractor,
    http::{HttpRequest, HttpResponse},
    response::IntoResponse,
    utils::ArrayMap,
};

pub(crate) struct ParsedParams(pub(crate) ArrayMap<String, String, 32>);

fn get_value<'req>(req: &'req mut HttpRequest, key: &'static str) -> Option<&'req String> {
    req.extensions
        .get::<ParsedParams>()
        .and_then(|parsed| parsed.0.get(key))
}

fn get_value_err<'req, const KEY: &'static str>(
    req: &'req mut HttpRequest,
) -> Result<&'req String, ParamMissingRejection<KEY>> {
    match get_value(req, KEY) {
        Some(v) => Ok(v),
        None => Err(ParamMissingRejection {}),
    }
}

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
    type Error = ParamMissingRejection<KEY>;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        match get_value_err::<KEY>(req) {
            Ok(value) => Ok(Self {
                value: value.clone(),
            }),
            Err(err) => Err(err),
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
    type Error = ParamMissingRejection<KEY>;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            value: get_value(req, KEY).cloned(),
        })
    }
}

pub struct ParamMissingRejection<const KEY: &'static str> {}

#[cfg(feature = "std")]
impl<const KEY: &'static str> IntoResponse for ParamMissingRejection<KEY> {
    fn into_response(self) -> HttpResponse {
        std::format!(
            "HTTP request URL parameters did not contain a value with the key `{}`",
            KEY,
        )
        .into_response()
    }
}

#[cfg(feature = "vfmt")]
impl<const KEY: &'static str> IntoResponse for ParamMissingRejection<KEY> {
    fn into_response(self) -> HttpResponse {
        vfmt::format!(
            "HTTP request URL parameters did not contain a value with the key `{}`",
            KEY,
        )
        .into_response()
    }
}

#[cfg(feature = "std")]
pub struct ParseParam<const KEY: &'static str, T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    value: T,
}

#[cfg(feature = "std")]
impl<const KEY: &'static str, T> const Deref for ParseParam<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[cfg(feature = "std")]
impl<const KEY: &'static str, T> const DerefMut for ParseParam<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[cfg(feature = "std")]
impl<const KEY: &'static str, T> Extractor for ParseParam<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        match get_value_err(req, KEY) {
            Ok(value) => match T::from_str(value) {
                Ok(value) => Ok(Self { value }),
                Err(err) => Err(InternalError::BadRequest(std::format!(
                    "HTTP request URL parameter with key `{}` could not be parsed: {:?}",
                    KEY,
                    err
                ))),
            },
            Err(err) => Err(err),
        }
    }
}

#[cfg(feature = "vfmt")]
pub struct ParseParam<const KEY: &'static str, T>
where
    T: FromStr,
    <T as FromStr>::Err: vfmt::uDebug,
{
    value: T,
}

#[cfg(feature = "vfmt")]
impl<const KEY: &'static str, T> const Deref for ParseParam<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: vfmt::uDebug,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

#[cfg(feature = "vfmt")]
impl<const KEY: &'static str, T> const DerefMut for ParseParam<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: vfmt::uDebug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[cfg(feature = "vfmt")]
impl<const KEY: &'static str, T> Extractor for ParseParam<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: vfmt::uDebug,
{
    type Error = HttpResponse;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        match get_value_err::<KEY>(req) {
            Ok(value) => match T::from_str(value) {
                Ok(value) => Ok(Self { value }),
                Err(err) => Err(ParamRejection::<KEY, T> { err }.into_response()),
            },
            Err(err) => Err(err.into_response()),
        }
    }
}

pub struct ParamRejection<const KEY: &'static str, T>
where
    T: FromStr,
{
    err: <T as FromStr>::Err,
}

#[cfg(feature = "std")]
impl<const KEY: &'static str, T> IntoResponse for ParamRejection<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    fn into_response(self) -> HttpResponse {
        std::format!(
            "HTTP request URL parameter with key `{}` could not be parsed: {:?}",
            KEY,
            self.err
        )
        .into_response()
    }
}

#[cfg(feature = "vfmt")]
impl<const KEY: &'static str, T> IntoResponse for ParamRejection<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: vfmt::uDebug,
{
    fn into_response(self) -> HttpResponse {
        vfmt::format!(
            "HTTP request URL parameter with key `{}` could not be parsed: {:?}",
            KEY,
            self.err
        )
        .into_response()
    }
}
