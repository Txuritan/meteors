use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use crate::{
    extractor::Extractor,
    http::{HttpRequest, HttpResponse},
    response::IntoResponse,
    utils::ArrayMap,
};

pub trait ParamKey {
    const KEY: &'static str;
}

pub(crate) struct ParsedParams(pub(crate) ArrayMap<String, String, 32>);

fn get_value<'req>(req: &'req mut HttpRequest, key: &'static str) -> Option<&'req String> {
    req.extensions
        .get::<ParsedParams>()
        .and_then(|parsed| parsed.0.get(key))
}

fn get_value_err<'req, KEY: ParamKey>(
    req: &'req mut HttpRequest,
) -> Result<&'req String, ParamMissingRejection<KEY>> {
    match get_value(req, KEY::KEY) {
        Some(v) => Ok(v),
        None => Err(ParamMissingRejection { _k: PhantomData }),
    }
}

pub struct Param<KEY: ParamKey> {
    value: String,
    _k: PhantomData<KEY>,
}

impl<KEY: ParamKey> const Deref for Param<KEY> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<KEY: ParamKey> const DerefMut for Param<KEY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<KEY: ParamKey> Extractor for Param<KEY> {
    type Error = ParamMissingRejection<KEY>;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        match get_value_err::<KEY>(req) {
            Ok(value) => Ok(Self {
                value: value.clone(),
                _k: PhantomData,
            }),
            Err(err) => Err(err),
        }
    }
}

pub struct OptionalParam<KEY: ParamKey> {
    value: Option<String>,
    _k: PhantomData<KEY>,
}

impl<KEY: ParamKey> const Deref for OptionalParam<KEY> {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<KEY: ParamKey> const DerefMut for OptionalParam<KEY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<KEY: ParamKey> Extractor for OptionalParam<KEY> {
    type Error = ParamMissingRejection<KEY>;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            value: get_value(req, KEY::KEY).cloned(),
            _k: PhantomData,
        })
    }
}

pub struct ParamMissingRejection<KEY: ParamKey> {
    _k: PhantomData<KEY>,
}

#[cfg(feature = "std")]
impl<KEY: ParamKey> IntoResponse for ParamMissingRejection<KEY> {
    fn into_response(self) -> HttpResponse {
        std::format!(
            "HTTP request URL parameters did not contain a value with the key `{}`",
            KEY::KEY,
        )
        .into_response()
    }
}

#[cfg(feature = "vfmt")]
impl<KEY: ParamKey> IntoResponse for ParamMissingRejection<KEY> {
    fn into_response(self) -> HttpResponse {
        vfmt::format!(
            "HTTP request URL parameters did not contain a value with the key `{}`",
            KEY::KEY,
        )
        .into_response()
    }
}

#[cfg(feature = "std")]
pub struct ParseParam<KEY: ParamKey, T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    value: T,
    _k: PhantomData<KEY>,
}

#[cfg(feature = "std")]
impl<KEY: ParamKey, T> const Deref for ParseParam<KEY, T>
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
impl<KEY: ParamKey, T> const DerefMut for ParseParam<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[cfg(feature = "std")]
impl<KEY: ParamKey, T> Extractor for ParseParam<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        match get_value_err(req, KEY::KEY) {
            Ok(value) => match T::from_str(value) {
                Ok(value) => Ok(Self {
                    value,
                    _k: PhantomData,
                }),
                Err(err) => Err(InternalError::BadRequest(std::format!(
                    "HTTP request URL parameter with key `{}` could not be parsed: {:?}",
                    KEY::KEY,
                    err
                ))),
            },
            Err(err) => Err(err),
        }
    }
}

#[cfg(feature = "vfmt")]
pub struct ParseParam<KEY: ParamKey, T>
where
    T: FromStr,
    <T as FromStr>::Err: vfmt::uDebug,
{
    value: T,
    _k: PhantomData<KEY>,
}

#[cfg(feature = "vfmt")]
impl<KEY: ParamKey, T> const Deref for ParseParam<KEY, T>
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
impl<KEY: ParamKey, T> const DerefMut for ParseParam<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: vfmt::uDebug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[cfg(feature = "vfmt")]
impl<KEY: ParamKey, T> Extractor for ParseParam<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: vfmt::uDebug,
{
    type Error = HttpResponse;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        match get_value_err::<KEY>(req) {
            Ok(value) => match T::from_str(value) {
                Ok(value) => Ok(Self {
                    value,
                    _k: PhantomData,
                }),
                Err(err) => Err(ParamRejection::<KEY, T> {
                    err,
                    _k: PhantomData,
                }
                .into_response()),
            },
            Err(err) => Err(err.into_response()),
        }
    }
}

pub struct ParamRejection<KEY: ParamKey, T>
where
    T: FromStr,
{
    err: <T as FromStr>::Err,
    _k: PhantomData<KEY>,
}

#[cfg(feature = "std")]
impl<KEY: ParamKey, T> IntoResponse for ParamRejection<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    fn into_response(self) -> HttpResponse {
        std::format!(
            "HTTP request URL parameter with key `{}` could not be parsed: {:?}",
            KEY::KEY,
            self.err
        )
        .into_response()
    }
}

#[cfg(feature = "vfmt")]
impl<KEY: ParamKey, T> IntoResponse for ParamRejection<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: vfmt::uDebug,
{
    fn into_response(self) -> HttpResponse {
        vfmt::format!(
            "HTTP request URL parameter with key `{}` could not be parsed: {:?}",
            KEY::KEY,
            self.err
        )
        .into_response()
    }
}
