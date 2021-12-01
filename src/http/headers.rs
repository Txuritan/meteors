use std::{borrow::Cow, cmp, fmt};

pub struct HttpHeaders {}

pub struct HttpHeader {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
