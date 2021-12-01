mod encoding;

pub mod headers;
pub mod uri;

mod status;

use std::{
    borrow::Cow,
    cmp, fmt,
    io::Read,
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

pub use self::{headers::HttpHeaderName, status::StatusCode};

#[derive(Debug)]
pub enum HttpError {
    InvalidRequest,

    ParseMissingMeta,
    ParseMetaMissingMethod,
    ParseMetaMissingUri,
    ParseMetaMissingVersion,

    ParseUnknownMethod,
    ParseUnknownVersion,

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

#[derive(Debug, PartialEq)]
pub enum HttpBody {
    None,
    Bytes(&'static [u8]),
    Vector(Vec<u8>),
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

pub struct HttpRequest2 {
    /// The request's method
    pub method: HttpMethod,
    /// The request's URI
    pub uri: HttpResource,
    /// The request's version
    pub version: HttpVersion,

    /// The request's headers
    pub headers: HttpHeaders,

    /// The request's extensions
    pub extensions: Extensions,

    /// The request's body
    pub body: HttpBody,
}

impl HttpRequest2 {
    pub fn from_buf_reader<T>(reader: &mut T) -> Result<HttpRequest2, HttpError>
    where
        T: BufRead,
    {
        let (method, uri, version) = Self::read_meta(reader)?;
        let headers = Self::read_headers(reader)?;
        let body = Self::read_body(reader, &headers)?;

        Ok(HttpRequest2 {
            method,
            version,
            uri,
            headers,
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
            todo!()
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
                todo!()
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

    pub fn read_body<T>(reader: &mut T, headers: &HttpHeaders) -> Result<HttpBody, HttpError>
    where
        T: BufRead,
    {
        match headers.get(&headers::CONTENT_LENGTH) {
            Some(len) => {
                let len = len.parse::<usize>()?;

                if len == 0 {
                    HttpBody::None
                } else {
                    match headers.get(&headers::TRANSFER_ENCODING) {
                        Some(value) if value.eq_ignore_ascii_case("chunked") => {
                            todo!()
                        }
                        Some(value) => todo!("handle `Transfer-Encoding` value `{}`", value),
                        None => HttpBody::None,
                    }
                }
            }
            None => HttpBody::None,
        };

        todo!()
    }
}

impl fmt::Debug for HttpRequest2 {
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

impl cmp::PartialEq for HttpRequest2 {
    fn eq(&self, other: &Self) -> bool {
        self.method == other.method
            && self.uri == other.uri
            && self.version == other.version
            && self.headers == other.headers
            && self.body == other.body
    }
}

#[derive(Debug, PartialEq)]
pub struct HttpHeaderData {
    pub method: HttpMethod,
    pub url: String,
    pub query: String,
    pub query_params: HttpParams,
    pub version: HttpVersion,
    pub headers: HttpHeaders,
}

pub struct HttpRequest {
    pub header_data: HttpHeaderData,
    pub body: Vec<u8>,

    pub params: HttpParams,

    pub data: Arc<Extensions>,

    pub extensions: Extensions,
}

pub struct HttpResponse {
    pub version: HttpVersion,
    pub status: StatusCode,
    pub headers: ArrayMap<headers::HttpHeaderName, String, 64>,
    pub body: HttpBody,
}

impl HttpResponse {
    #[allow(clippy::new_ret_no_self)]
    pub const fn new(status: StatusCode) -> Self {
        Self {
            version: HttpVersion::Http10,
            status,
            headers: ArrayMap::new(),
            body: HttpBody::None,
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

    pub const fn status(mut self, status: StatusCode) -> Self {
        self.status = status;

        self
    }

    pub fn header<V>(mut self, key: headers::HttpHeaderName, value: V) -> Self
    where
        V: ToString,
    {
        self.headers.insert(key, value.to_string());

        self
    }

    pub fn body<B>(mut self, body: B) -> HttpResponse
    where
        B: Into<HttpBody>,
    {
        self.body = body.into();

        self
    }
}

pub fn read_request<R>(reader: &mut R) -> Result<(HttpHeaderData, Vec<u8>), HttpError>
where
    R: Read,
{
    struct State {
        data: Vec<u8>,
        total_read: usize,
        amount_read: usize,
        read_buffer: [u8; BUFFER_SIZE],
    }

    fn double_newline(bytes: &[u8]) -> bool {
        bytes == &b"\r\n\r\n"[..]
    }

    const BUFFER_SIZE: usize = 512;
    const MAX_BYTES: usize = 1028 * 8;

    let mut state = State {
        data: Vec::with_capacity(512),
        total_read: 0,
        amount_read: BUFFER_SIZE,
        read_buffer: [0; BUFFER_SIZE],
    };

    loop {
        if state.amount_read == 0 {
            break;
        }

        state.amount_read = reader.read(&mut state.read_buffer)?;

        if state.amount_read == 0 {
            break;
        }

        (state.total_read) += state.amount_read;

        state
            .data
            .extend_from_slice(&state.read_buffer[..state.amount_read]);

        (state.read_buffer) = [0; BUFFER_SIZE];

        if state.data.windows(4).any(double_newline) {
            break;
        }

        if state.total_read >= MAX_BYTES {
            break;
        }
    }

    let header_bytes = if let Some(i) = state.data.windows(4).position(double_newline) {
        &state.data[..(i + 2)]
    } else {
        &state.data[..]
    };

    let header_str = String::from_utf8_lossy(header_bytes);

    let header_data = parse_header(header_str.as_ref())?;

    let (header_data, body) =
        if let Some(header) = header_data.headers.get(&headers::CONTENT_LENGTH) {
            let amount_of_bytes = header.trim().parse::<usize>()?;

            let left_to_read = MAX_BYTES.saturating_sub(state.total_read);

            if amount_of_bytes >= left_to_read {
                (header_data, vec![])
            } else {
                let mut body = Vec::with_capacity(left_to_read.min(amount_of_bytes));

                reader
                    .by_ref()
                    .take(amount_of_bytes as u64)
                    .read_to_end(&mut body)?;

                (header_data, Vec::from(&body[..]))
            }
        } else {
            (header_data, vec![])
        };

    Ok((header_data, body))
}

pub fn parse_header(headers: &str) -> Result<HttpHeaderData, HttpError> {
    let mut lines = headers.lines();

    let meta = lines.next().ok_or(HttpError::ParseMissingMeta)?;

    let (method, url, query, query_params, version) = {
        let mut meta_parts = meta.split(' ').filter(|part| !part.is_empty());

        let method = HttpMethod::from_str(
            meta_parts
                .next()
                .ok_or(HttpError::ParseMetaMissingMethod)?
                .trim(),
        )?;

        let url = meta_parts
            .next()
            .ok_or(HttpError::ParseMetaMissingUri)?
            .trim();

        let (url, query) = url.split_at(url.find('?').unwrap_or_else(|| url.len()));

        let mut query_params = HttpParams::new();

        for (key, value) in
            crate::http::encoding::form::parse(query.trim_start_matches('?').as_bytes())
        {
            query_params.insert(key.to_string(), value.to_string());
        }

        let version = HttpVersion::from_str(
            meta_parts
                .next()
                .ok_or(HttpError::ParseMetaMissingVersion)?
                .trim(),
        )?;

        (
            method,
            url.to_string(),
            query.to_string(),
            query_params,
            version,
        )
    };

    let headers = {
        let mut headers = HttpHeaders::new();

        for header in &mut lines {
            if header.is_empty() {
                break;
            }

            if let Some(idx) = header.find(':') {
                let (key, value) = header.split_at(idx);

                headers.insert(
                    headers::HttpHeaderName(Cow::Owned(key.trim().to_string())),
                    value.trim_start_matches(": ").trim().to_string(),
                );
            }
        }

        headers
    };

    Ok(HttpHeaderData {
        method,
        url,
        query,
        query_params,
        version,
        headers,
    })
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
        write!(stream, "{}: {}\r\n", key, value)?;
    }

    fn write_bytes(
        headers: &ArrayMap<headers::HttpHeaderName, String, 64>,
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
        HttpBody::None => {
            write!(stream, "Content-Length: 0\r\n")?;
        }
        HttpBody::Bytes(bytes) => {
            write_bytes(&res.headers, bytes, compress, stream)?;
        }
        HttpBody::Vector(bytes) => {
            write_bytes(&res.headers, bytes.as_slice(), compress, stream)?;
        }
    }

    Ok(())
}
