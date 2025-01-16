use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use crate::{
    extractor::Extractor,
    http::{HttpRequest, HttpResponse},
    response::IntoResponse,
};

pub trait HeaderKey {
    const KEY: &'static str;
}

fn get_value<'req>(req: &'req HttpRequest, key: &'static str) -> Option<&'req String> {
    req.headers
        .iter()
        .find(|(k, _)| k.0.eq_ignore_ascii_case(key))
        .map(|(_, value)| value)
}

fn get_value_err<'req, KEY: HeaderKey>(
    req: &'req HttpRequest,
) -> Result<&'req String, HeaderMissingRejection<KEY>> {
    match get_value(req, KEY::KEY) {
        Some(v) => Ok(v),
        None => Err(HeaderMissingRejection { _k: PhantomData }),
    }
}

pub struct Header<KEY: HeaderKey> {
    value: String,
    _k: PhantomData<KEY>,
}

impl<KEY: HeaderKey> const Deref for Header<KEY> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<KEY: HeaderKey> const DerefMut for Header<KEY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<KEY: HeaderKey> Extractor for Header<KEY> {
    type Error = HeaderMissingRejection<KEY>;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        match get_value_err::<KEY>(&*req) {
            Ok(value) => Ok(Self {
                value: value.clone(),
                _k: PhantomData,
            }),
            Err(err) => Err(err),
        }
    }
}

pub struct OptionalHeader<KEY: HeaderKey> {
    value: Option<String>,
    _k: PhantomData<KEY>,
}

impl<KEY: HeaderKey> const Deref for OptionalHeader<KEY> {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<KEY: HeaderKey> const DerefMut for OptionalHeader<KEY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<KEY: HeaderKey> Extractor for OptionalHeader<KEY> {
    type Error = HeaderMissingRejection<KEY>;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            value: get_value(&*req, KEY::KEY).cloned(),
            _k: PhantomData,
        })
    }
}

pub struct HeaderMissingRejection<KEY: HeaderKey> {
    _k: PhantomData<KEY>,
}

#[cfg(feature = "std")]
impl<KEY: HeaderKey> IntoResponse for HeaderMissingRejection<KEY> {
    fn into_response(self) -> HttpResponse {
        std::format!(
            "HTTP request header with key `{}` could not be found",
            KEY::KEY,
        )
        .into_response()
    }
}

#[cfg(feature = "vfmt")]
impl<KEY: HeaderKey> IntoResponse for HeaderMissingRejection<KEY> {
    fn into_response(self) -> HttpResponse {
        vfmt::format!(
            "HTTP request header with key `{}` could not be found",
            KEY::KEY,
        )
        .into_response()
    }
}

#[cfg(feature = "std")]
pub struct ParseHeader<KEY: HeaderKey, T>(pub T)
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug;

#[cfg(feature = "std")]
impl<KEY: HeaderKey, T> Extractor for ParseHeader<KEY, T>
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
                    KEY::KEY,
                    err
                ))),
            },
            Err(err) => Err(err),
        }
    }
}

#[cfg(feature = "vfmt")]
pub struct ParseHeader<KEY: HeaderKey, T>
where
    T: FromStr,
    <T as FromStr>::Err: vfmt::uDebug,
{
    value: T,
    _k: PhantomData<KEY>,
}

#[cfg(feature = "vfmt")]
impl<KEY: HeaderKey, T> Extractor for ParseHeader<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: vfmt::uDebug,
{
    type Error = HttpResponse;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        match get_value_err::<KEY>(&*req) {
            Ok(value) => match T::from_str(value) {
                Ok(value) => Ok(Self {
                    value,
                    _k: PhantomData,
                }),
                Err(err) => Err(HeaderRejection::<KEY, T> {
                    err,
                    _k: PhantomData,
                }
                .into_response()),
            },
            Err(err) => Err(err.into_response()),
        }
    }
}

pub struct HeaderRejection<KEY: HeaderKey, T: FromStr> {
    err: <T as FromStr>::Err,
    _k: PhantomData<KEY>,
}

#[cfg(feature = "std")]
impl<KEY: HeaderKey, T> IntoResponse for HeaderRejection<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    fn into_response(self) -> HttpResponse {
        std::format!(
            "HTTP request header with key `{}` could not be parsed: {:?}",
            KEY::KEY,
            self.err
        )
        .into_response()
    }
}

#[cfg(feature = "vfmt")]
impl<KEY: HeaderKey, T> IntoResponse for HeaderRejection<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: vfmt::uDebug,
{
    fn into_response(self) -> HttpResponse {
        vfmt::format!(
            "HTTP request header with key `{}` could not be parsed: {:?}",
            KEY::KEY,
            self.err
        )
        .into_response()
    }
}
