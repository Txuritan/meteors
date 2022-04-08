mod status;

pub mod encoding;

pub mod headers;
pub mod uri;

use std::{
    cmp, fmt,
    io::{BufRead, Write},
    net::TcpStream,
    str::FromStr,
    sync::Arc,
};

use crate::{
    extensions::Extensions,
    http::uri::HttpResource,
    utils::{ArrayMap, Ascii, Const},
};

#[doc(inline)]
pub use self::{
    headers::{HttpHeaderMap, HttpHeaderName, HttpHeaderValue},
    status::StatusCode,
};

#[derive(Debug)]
pub enum HttpError {
    InvalidRequest,

    ParseMissingMeta,
    ParseMetaMissingMethod,
    ParseMetaMissingUri,
    ParseMetaMissingVersion,

    ParseUnknownMethod,
    ParseUnknownVersion,

    ZeroBytesRead,

    Io(std::io::Error),
    ParseInt(std::num::ParseIntError),
}

impl const From<std::io::Error> for HttpError {
    fn from(v: std::io::Error) -> Self {
        Self::Io(v)
    }
}

impl const From<std::num::ParseIntError> for HttpError {
    fn from(v: std::num::ParseIntError) -> Self {
        Self::ParseInt(v)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpBody {
    Bytes(&'static [u8]),
    Vector(Vec<u8>),
}

impl HttpBody {
    pub fn as_vec(&self) -> Vec<u8> {
        match self {
            HttpBody::Bytes(bytes) => bytes.to_vec(),
            HttpBody::Vector(vector) => vector.clone(),
        }
    }
}

impl const From<&'static str> for HttpBody {
    fn from(s: &'static str) -> Self {
        Self::Bytes(s.as_bytes())
    }
}

impl const From<&'static [u8]> for HttpBody {
    fn from(s: &'static [u8]) -> Self {
        Self::Bytes(s)
    }
}

impl From<String> for HttpBody {
    fn from(s: String) -> Self {
        Self::Vector(s.as_bytes().to_vec())
    }
}

impl const From<Vec<u8>> for HttpBody {
    fn from(s: Vec<u8>) -> Self {
        Self::Vector(s)
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum HttpMethod {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
}

impl FromStr for HttpMethod {
    type Err = HttpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Self::Get),
            "HEAD" => Ok(Self::Head),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "CONNECT" => Ok(Self::Connect),
            "OPTIONS" => Ok(Self::Options),
            "TRACE" => Ok(Self::Trace),
            "PATCH" => Ok(Self::Patch),
            _ => Err(HttpError::ParseUnknownMethod),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum HttpVersion {
    Http09,
    Http10,
    Http11,
}

impl fmt::Display for HttpVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http09 => write!(f, "HTTP/0.9"),
            Self::Http10 => write!(f, "HTTP/1.0"),
            Self::Http11 => write!(f, "HTTP/1.1"),
        }
    }
}

impl FromStr for HttpVersion {
    type Err = HttpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/0.9" => Ok(Self::Http09),
            "HTTP/1.0" => Ok(Self::Http10),
            "HTTP/1.1" => Ok(Self::Http11),
            _ => Err(HttpError::ParseUnknownVersion),
        }
    }
}

pub type HttpHeaders = ArrayMap<headers::HttpHeaderName, String, 32>;
pub type HttpParams = ArrayMap<String, String, 32>;

pub struct HttpRequest {
    /// The request's method
    pub method: HttpMethod,
    /// The request's URI
    pub uri: HttpResource,
    /// The request's version
    pub version: HttpVersion,

    /// The request's headers
    pub headers: HttpHeaders,

    /// The server's extensions
    pub data: Arc<Extensions>,
    /// The request's extensions
    pub extensions: Extensions,

    /// The request's body
    pub body: Option<HttpBody>,
}

impl HttpRequest {
    pub fn from_buf_reader<T>(data: Arc<Extensions>, reader: &mut T) -> Result<Self, HttpError>
    where
        T: BufRead,
    {
        let (method, uri, version) = Self::read_meta(reader)?;
        let headers = Self::read_headers(reader)?;
        let body = Self::read_body(reader, &headers)?;

        Ok(Self {
            method,
            version,
            uri,
            headers,
            data,
            extensions: Extensions::new(),
            body,
        })
    }

    pub fn read_meta<T>(
        reader: &mut T,
    ) -> Result<(HttpMethod, HttpResource, HttpVersion), HttpError>
    where
        T: BufRead,
    {
        use crate::utils::StringExt as _;

        let mut buffer = Vec::with_capacity(256);

        let len = reader.read_until(b'\n', &mut buffer)?;
        if len == 0 {
            // TODO: maybe try and read a few times to see if it timed out
            return Err(HttpError::ZeroBytesRead);
        }

        let mut offset = 0;

        let raw_method = Ascii::read_until(&buffer, &mut offset, b' ')
            .ok_or(HttpError::ParseMetaMissingMethod)?;
        let method = HttpMethod::from_str(&*raw_method)?;

        offset += 1;

        let raw_uri =
            Ascii::read_until(&buffer, &mut offset, b' ').ok_or(HttpError::ParseMetaMissingUri)?;
        let uri = HttpResource::new(&raw_uri);

        offset += 1;

        let mut raw_version = Ascii::read_until(&buffer, &mut offset, b'\n')
            .ok_or(HttpError::ParseMetaMissingVersion)?;
        raw_version.trim();
        let version = HttpVersion::from_str(&*raw_version)?;

        Ok((method, uri, version))
    }

    pub fn read_headers<T>(reader: &mut T) -> Result<HttpHeaders, HttpError>
    where
        T: BufRead,
    {
        let mut headers = HttpHeaders::new();

        let mut buffer = Vec::with_capacity(256);

        loop {
            if !buffer.is_empty() {
                buffer.clear();
            }

            let len = reader.read_until(b'\n', &mut buffer)?;
            if len == 0 {
                // TODO: maybe try and read a few times to see if it timed out
                return Err(HttpError::ZeroBytesRead);
            }

            if buffer == b"\r\n" {
                break;
            }

            if let Some(colon_index) = Ascii::find_index(&buffer, b':') {
                if colon_index > buffer.len() {
                    continue;
                }

                // TODO(txuritan): use `slice::split_at_unchecked` when the api becomes public
                let (head, tail) = unsafe {
                    (
                        Const::slice_range_get_unchecked(&buffer, 0..colon_index),
                        Const::slice_range_get_unchecked(&buffer, colon_index..(buffer).len()),
                    )
                };

                let tail = match tail {
                    [b':', b' ', tail @ .., b'\r', b'\n'] => tail,
                    tail => tail,
                };

                // TODO(txuritan): maybe see about propagating this error
                let mut head = match String::from_utf8(head.to_vec()).ok() {
                    Some(it) => it,
                    None => continue,
                };
                let mut tail = match String::from_utf8(tail.to_vec()).ok() {
                    Some(it) => it,
                    None => continue,
                };

                {
                    use crate::utils::StringExt as _;

                    head.trim();
                    tail.trim();
                }

                headers.insert(HttpHeaderName(head.into()), tail);
            }
        }

        Ok(headers)
    }

    pub fn read_body<T>(
        reader: &mut T,
        headers: &HttpHeaders,
    ) -> Result<Option<HttpBody>, HttpError>
    where
        T: BufRead,
    {
        match headers.get(&headers::CONTENT_LENGTH) {
            Some(len) => {
                let len = len.parse::<usize>()?;

                if len == 0 {
                    Ok(None)
                } else {
                    // TODO(txuritan): limit the amount of bytes that can be read
                    let mut content = vec![0; len];

                    reader.read_exact(&mut content[0..len])?;

                    Ok(Some(HttpBody::Vector(content)))
                }
            }
            None => match headers.get(&headers::TRANSFER_ENCODING) {
                Some(value) if value.eq_ignore_ascii_case("chunked") => {
                    todo!()
                }
                Some(value) => todo!("handle `Transfer-Encoding` value `{}`", value),
                None => Ok(None),
            },
        }
    }
}

impl fmt::Debug for HttpRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HttpRequest2")
            .field("method", &self.method)
            .field("uri", &self.uri)
            .field("version", &self.version)
            .field("headers", &self.headers)
            .field("body", &self.body)
            .finish()
    }
}

impl cmp::PartialEq for HttpRequest {
    fn eq(&self, other: &Self) -> bool {
        self.method == other.method
            && self.uri == other.uri
            && self.version == other.version
            && self.headers == other.headers
            && self.body == other.body
    }
}

pub struct HttpResponse {
    pub version: HttpVersion,
    pub status: StatusCode,
    pub extensions: Extensions,
    pub headers: headers::HttpHeaderMap,
    pub body: Option<HttpBody>,
}

impl HttpResponse {
    pub const fn new(status: StatusCode) -> Self {
        Self {
            version: HttpVersion::Http10,
            status,
            extensions: Extensions::new(),
            headers: headers::HttpHeaderMap::new(),
            body: None,
        }
    }

    pub const fn ok() -> Self {
        Self::new(StatusCode::OK)
    }

    pub const fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND)
    }

    pub const fn internal_server_error() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub const fn bad_request() -> Self {
        Self::new(StatusCode::BAD_REQUEST)
    }

    pub const fn status(&self) -> StatusCode {
        self.status
    }

    pub const fn status_mut(&mut self) -> &mut StatusCode {
        &mut self.status
    }

    pub const fn headers(&self) -> &headers::HttpHeaderMap {
        &self.headers
    }

    pub const fn headers_mut(&mut self) -> &mut headers::HttpHeaderMap {
        &mut self.headers
    }

    pub const fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    pub const fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }

    pub fn header<V>(mut self, key: headers::HttpHeaderName, value: V) -> Self
    where
        V: ToString,
    {
        self.headers
            .insert(key, headers::HttpHeaderValue::new(value.to_string()));

        self
    }

    pub fn body<B>(mut self, body: B) -> HttpResponse
    where
        B: Into<HttpBody>,
    {
        self.body = Some(body.into());

        self
    }
}

pub fn write_response(
    res: HttpResponse,
    compress: bool,
    stream: &mut TcpStream,
) -> std::io::Result<()> {
    write!(
        stream,
        "{} {} {}\r\n",
        res.version,
        res.status.0,
        res.status.phrase()
    )?;

    for (key, value) in &res.headers {
        write!(stream, "{}: {}\r\n", key, value.as_str())?;
    }

    fn write_bytes(
        headers: &headers::HttpHeaderMap,
        bytes: &[u8],
        compress: bool,
        stream: &mut TcpStream,
    ) -> std::io::Result<()> {
        let pre_compressed = {
            match headers.get(&headers::CONTENT_ENCODING) {
                Some(header) => matches!(header.as_str(), "deflate" | "gzip"),
                None => false,
            }
        };

        if compress && !pre_compressed {
            write!(stream, "Content-Encoding: deflate\r\n")?;

            let compressed = miniz_oxide::deflate::compress_to_vec(bytes, 8);

            write!(stream, "Content-Length: {}\r\n", compressed.len())?;

            write!(stream, "\r\n")?;

            stream.write_all(&compressed)?;
        } else {
            write!(stream, "Content-Length: {}\r\n", bytes.len())?;

            write!(stream, "\r\n")?;

            stream.write_all(bytes)?;
        }

        Ok(())
    }

    match res.body {
        Some(HttpBody::Bytes(bytes)) => {
            write_bytes(&res.headers, bytes, compress, stream)?;
        }
        Some(HttpBody::Vector(bytes)) => {
            write_bytes(&res.headers, bytes.as_slice(), compress, stream)?;
        }
        None => {
            write!(stream, "Content-Length: 0\r\n")?;
        }
    }

    Ok(())
}
