use std::{
    ops::{Deref, DerefMut},
    str::FromStr,
};

use crate::{
    error::InternalError,
    extractor::Extractor,
    http::{encoding::percent::percent_decode, HttpRequest},
    utils::ArrayMap,
    Error,
};

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

fn get_value_err<'req>(
    req: &'req mut HttpRequest,
    key: &'static str,
) -> Result<&'req String, Error> {
    match get_value(req, key) {
        Some(v) => Ok(v),
        None => Err(InternalError::BadRequest(crate::wrapper::format!(
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
        match get_value_err(req, KEY) {
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
            value: get_value(req, KEY).cloned(),
        })
    }
}

pub struct ParseQuery<const KEY: &'static str, T>
where
    T: FromStr,
    <T as FromStr>::Err: crate::wrapper::Debug,
{
    value: T,
}

impl<const KEY: &'static str, T> const Deref for ParseQuery<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: crate::wrapper::Debug,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str, T> const DerefMut for ParseQuery<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: crate::wrapper::Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<const KEY: &'static str, T> Extractor for ParseQuery<KEY, T>
where
    T: FromStr,
    <T as FromStr>::Err: crate::wrapper::Debug,
{
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        match get_value_err(req, KEY) {
            Ok(value) => match T::from_str(value) {
                Ok(value) => Ok(Self { value }),
                Err(err) => Err(InternalError::BadRequest(crate::wrapper::format!(
                    "HTTP request URL query with key `{}` could not be parsed: {:?}",
                    KEY,
                    err
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
            value: match &req.uri.query {
                Some(query) => Clone::clone(query),
                None => String::new(),
            },
        })
    }
}
