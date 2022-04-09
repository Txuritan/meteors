use std::{borrow::Cow, cmp, convert::TryFrom, fmt};

use crate::utils::{array_map, ArrayMap};

pub struct HttpHeaderMap {
    inner: ArrayMap<HttpHeaderName, HttpHeaderValue, 64>,
}

impl HttpHeaderMap {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: ArrayMap::new(),
        }
    }

    #[inline]
    pub fn get(&self, key: &HttpHeaderName) -> Option<&HttpHeaderValue> {
        self.inner.get(key)
    }

    #[inline]
    pub fn insert(&mut self, key: HttpHeaderName, value: HttpHeaderValue) {
        self.inner.insert(key, value);
    }

    #[inline]
    pub(crate) fn iter(&self) -> array_map::Iter<'_, HttpHeaderName, HttpHeaderValue, 64> {
        self.inner.iter()
    }
}

impl<'m> IntoIterator for &'m HttpHeaderMap {
    type Item = (&'m HttpHeaderName, &'m HttpHeaderValue);

    type IntoIter = array_map::Iter<'m, HttpHeaderName, HttpHeaderValue, 64>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

pub struct HttpHeaderValue(Cow<'static, str>);

impl HttpHeaderValue {
    pub(crate) fn new(value: String) -> Self {
        Self(Cow::Owned(value))
    }

    pub(crate) fn new_static(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&'static str> for HttpHeaderValue {
    type Error = std::convert::Infallible;

    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        Ok(Self::new_static(value))
    }
}

impl TryFrom<String> for HttpHeaderValue {
    type Error = std::convert::Infallible;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self::new(value))
    }
}

#[derive(Debug, Clone, Eq, PartialOrd, Ord)]
pub struct HttpHeaderName(pub(crate) Cow<'static, str>);

impl HttpHeaderName {
    pub const fn new(key: &'static str) -> Self {
        Self(Cow::Borrowed(key))
    }
}

impl fmt::Display for HttpHeaderName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl cmp::PartialEq<str> for HttpHeaderName {
    fn eq(&self, other: &str) -> bool {
        self.0.to_lowercase() == other.to_lowercase()
    }
}

impl cmp::PartialEq<HttpHeaderName> for HttpHeaderName {
    fn eq(&self, other: &HttpHeaderName) -> bool {
        self.0.to_lowercase() == other.0.to_lowercase()
    }
}

macro_rules! impl_headers {
    ($( $name:ident => $phrase:expr , )*) => {
        $(
            pub const $name: HttpHeaderName = HttpHeaderName(Cow::Borrowed($phrase));
        )*
    }
}

impl_headers! {
    ACCEPT => "Accept",
    ACCEPT_CHARSET => "Accept-Charset",
    ACCEPT_ENCODING => "Accept-Encoding",
    ACCEPT_LANGUAGE => "Accept-Language",
    ACCEPT_RANGES => "Accept-Ranges",
    ACCESS_CONTROL_ALLOW_CREDENTIALS => "Access-Control-Allow-Credentials",
    ACCESS_CONTROL_ALLOW_HEADERS => "Access-Control-Allow-Headers",
    ACCESS_CONTROL_ALLOW_METHODS => "Access-Control-Allow-Methods",
    ACCESS_CONTROL_ALLOW_ORIGIN => "Access-Control-Allow-Origin",
    ACCESS_CONTROL_EXPOSE_HEADERS => "Access-Control-Expose-Headers",
    ACCESS_CONTROL_MAX_AGE => "Access-Control-Max-Age",
    ACCESS_CONTROL_REQUEST_HEADERS => "Access-Control-Request-Headers",
    ACCESS_CONTROL_REQUEST_METHOD => "Access-Control-Request-Method",
    AGE => "Age",
    ALLOW => "Allow",
    AUTHORIZATION => "Authorization",
    CACHE_CONTROL => "Cache-Control",
    CLEAR_SITE_DATA => "Clear-Site-Data",
    CONNECTION => "Connection",
    CONTENT_ENCODING => "Content-Encoding",
    CONTENT_LANGUAGE => "Content-Language",
    CONTENT_LENGTH => "Content-Length",
    CONTENT_LOCATION => "Content-Location",
    CONTENT_MD5 => "Content-MD5",
    CONTENT_RANGE => "Content-Range",
    CONTENT_TYPE => "Content-Type",
    COOKIE => "Cookie",
    DATE => "Date",
    ETAG => "ETag",
    EXPECT => "Expect",
    EXPIRES => "Expires",
    FORWARDED => "Forwarded",
    FROM => "From",
    HOST => "Host",
    IF_MATCH => "If-Match",
    IF_MODIFIED_SINCE => "If-Modified-Since",
    IF_NONE_MATCH => "If-None-match",
    IF_RANGE => "If-Range",
    IF_UNMODIFIED_SINCE => "If-Unmodified-Since",
    KEEP_ALIVE => "Keep-Alive",
    LAST_MODIFIED => "Last-Modified",
    LOCATION => "Location",
    MAX_FORWARDS => "Max-Forwards",
    ORIGIN => "Origin",
    PRAGMA => "Pragma",
    PROXY_AUTHENTICATE => "Proxy-Authenticate",
    PROXY_AUTHORIZATION => "Proxy-Authorization",
    PROXY_CONNECTION => "Proxy-Connection",
    REFERER => "Referer",
    RETRY_AFTER => "Retry-After",
    SERVER => "Server",
    SERVER_TIMING => "Server-Timing",
    SET_COOKIE => "Set-Cookie",
    SOURCE_MAP => "SourceMap",
    TE => "TE",
    TIMING_ALLOW_ORIGIN => "Timing-Allow-Origin",
    TRACEPARENT => "Traceparent",
    TRAILER => "Trailer",
    TRANSFER_ENCODING => "Transfer-Encoding",
    UPGRADE => "Upgrade",
    USER_AGENT => "User-Agent",
    VARY => "Vary",
    VIA => "Via",
    WARNING => "Warning",
    WWW_AUTHENTICATE => "WWW-Authenticate",
}
