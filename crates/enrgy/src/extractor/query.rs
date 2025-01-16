use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use crate::{
    extractor::Extractor,
    http::{encoding::percent::percent_decode, HttpRequest, HttpResponse},
    response::IntoResponse,
    utils::ArrayMap,
};

pub trait QueryKey {
    const KEY: &'static str;
}

struct ParsedQuery(ArrayMap<String, Option<String>, 32>);

// taken from [qstring](https://github.com/algesten/qstring)
// by Martin Algesten (algesten) - MIT License
fn str_to_map(origin: &str) -> ArrayMap<String, Option<String>, 32> {
    fn decode(s: &str) -> String {
        percent_decode(s.as_bytes())
            .decode_utf8()
            .map(|cow| cow.into_owned())
            .unwrap_or_else(|_| s.to_string())
    }

    // current slice left to find params in
    let mut cur = origin;

    // move forward if start with ?
    if !cur.is_empty() && &cur[0..1] == "?" {
        cur = &cur[1..];
    }

    // where we build found parameters into
    let mut params = ArrayMap::new();

    while !cur.is_empty() {
        // if we're positioned on a &, skip it
        if &cur[0..1] == "&" {
            cur = &cur[1..];

            continue;
        }

        // find position of next =
        let (name, rest) = match cur.find('=') {
            // no next =, name will be until next & or until end
            None => match cur.find('&') {
                // no &, name is until end
                None => {
                    params.insert(decode(cur), None);

                    break;
                }
                // name is until next &, which means no value and shortcut
                // to start straight after the &.
                Some(pos) => {
                    params.insert(decode(&cur[..pos]), None);
                    cur = &cur[(pos + 1)..];

                    continue;
                }
            },
            Some(pos) => {
                if let Some(apos) = cur.find('&') {
                    if apos < pos {
                        params.insert(decode(&cur[..apos]), None);
                        cur = &cur[(apos + 1)..];

                        continue;
                    }
                }

                (&cur[..pos], &cur[(pos + 1)..])
            }
        };
        // skip parameters with no name
        if name.is_empty() {
            cur = rest;

            continue;
        }

        // from rest, find next occurence of &
        let (value, newcur) = match rest.find('&') {
            // no next &, then value is all up until end
            None => (rest, ""),
            // found one, value is up until & and next round starts after.
            Some(pos) => (&rest[..pos], &rest[(pos + 1)..]),
        };

        // found a parameter
        params.insert(decode(name), Some(decode(value)));
        cur = newcur;
    }

    params
}

fn get_value<'req>(req: &'req mut HttpRequest, key: &'static str) -> Option<&'req String> {
    let query = req.uri.query.as_deref()?;

    if req.extensions.get::<ParsedQuery>().is_none() {
        req.extensions.insert(ParsedQuery(str_to_map(query)));
    }

    req.extensions
        .get::<ParsedQuery>()
        .and_then(|parsed| parsed.0.get(key).and_then(Option::as_ref))
}

fn get_value_err<'req, KEY: QueryKey>(
    req: &'req mut HttpRequest,
) -> Result<&'req String, QueryMissingRejection<KEY>> {
    match get_value(req, KEY::KEY) {
        Some(v) => Ok(v),
        None => Err(QueryMissingRejection::<KEY> { _k: PhantomData }),
    }
}

pub struct Query<KEY: QueryKey> {
    value: String,
    _k: PhantomData<KEY>,
}

impl<KEY: QueryKey> Deref for Query<KEY> {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<KEY: QueryKey> DerefMut for Query<KEY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<KEY: QueryKey> Extractor for Query<KEY> {
    type Error = QueryMissingRejection<KEY>;

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

pub struct OptionalQuery<KEY: QueryKey> {
    value: Option<String>,
    _k: PhantomData<KEY>,
}

impl<KEY: QueryKey> Deref for OptionalQuery<KEY> {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<KEY: QueryKey> DerefMut for OptionalQuery<KEY> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<KEY: QueryKey> Extractor for OptionalQuery<KEY> {
    type Error = QueryMissingRejection<KEY>;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            value: get_value(req, KEY::KEY).cloned(),
            _k: PhantomData,
        })
    }
}

pub struct QueryMissingRejection<KEY: QueryKey> {
    _k: PhantomData<KEY>,
}

#[cfg(feature = "std")]
impl<KEY: QueryKey> IntoResponse for QueryMissingRejection<KEY> {
    fn into_response(self) -> HttpResponse {
        std::format!(
            "HTTP request URL query did not contain a value with the key `{}`",
            KEY,
        )
        .into_response()
    }
}

#[cfg(feature = "vfmt")]
impl<KEY: QueryKey> IntoResponse for QueryMissingRejection<KEY> {
    fn into_response(self) -> HttpResponse {
        vfmt::format!(
            "HTTP request URL query did not contain a value with the key `{}`",
            KEY::KEY,
        )
        .into_response()
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
    type Error = RawQueryMissingRejection;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        let value = match &req.uri.query {
            Some(query) => Clone::clone(query),
            None => return Err(RawQueryMissingRejection {}),
        };

        Ok(Self { value })
    }
}

pub struct RawQueryMissingRejection {}

#[cfg(feature = "std")]
impl IntoResponse for RawQueryMissingRejection {
    fn into_response(self) -> HttpResponse {
        "HTTP request URL did not contain a query".into_response()
    }
}

#[cfg(feature = "vfmt")]
impl IntoResponse for RawQueryMissingRejection {
    fn into_response(self) -> HttpResponse {
        "HTTP request URL did not contain a query".into_response()
    }
}

#[cfg(feature = "std")]
pub struct ParseQuery<KEY: QueryKey, T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    value: T,
}

#[cfg(feature = "std")]
impl<KEY: QueryKey, T> const Deref for ParseQuery<KEY, T>
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
impl<KEY: QueryKey, T> const DerefMut for ParseQuery<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[cfg(feature = "std")]
impl<KEY: QueryKey, T> Extractor for ParseQuery<KEY, T>
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
                    "HTTP request URL query with key `{}` could not be parsed: {:?}",
                    KEY,
                    err
                ))),
            },
            Err(err) => Err(err),
        }
    }
}

#[cfg(feature = "vfmt")]
pub struct ParseQuery<KEY: QueryKey, T>
where
    T: FromStr,
    <T as FromStr>::Err: vfmt::uDebug,
{
    value: T,
    _k: PhantomData<KEY>,
}

#[cfg(feature = "vfmt")]
impl<KEY: QueryKey, T> const Deref for ParseQuery<KEY, T>
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
impl<KEY: QueryKey, T> const DerefMut for ParseQuery<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: vfmt::uDebug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

#[cfg(feature = "vfmt")]
impl<KEY: QueryKey, T> Extractor for ParseQuery<KEY, T>
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
                Err(err) => Err(QueryRejection::<KEY, T> {
                    err,
                    _k: PhantomData,
                }
                .into_response()),
            },
            Err(err) => Err(err.into_response()),
        }
    }
}

pub struct QueryRejection<KEY: QueryKey, T>
where
    T: FromStr,
{
    err: <T as FromStr>::Err,
    _k: PhantomData<KEY>,
}

#[cfg(feature = "std")]
impl<KEY: QueryKey, T> IntoResponse for QueryRejection<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Debug,
{
    fn into_response(self) -> HttpResponse {
        std::format!(
            "HTTP request URL query with key `{}` could not be parsed: {:?}",
            KEY,
            self.err
        )
        .into_response()
    }
}

#[cfg(feature = "vfmt")]
impl<KEY: QueryKey, T> IntoResponse for QueryRejection<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: vfmt::uDebug,
{
    fn into_response(self) -> HttpResponse {
        vfmt::format!(
            "HTTP request URL query with key `{}` could not be parsed: {:?}",
            KEY::KEY,
            self.err
        )
        .into_response()
    }
}
