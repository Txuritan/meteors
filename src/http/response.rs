use {
    crate::{
        http::{
            headers::{HeaderName, CONTENT_ENCODING},
            Body, StatusCode, Version,
        },
        utils::ArrayMap,
    },
    std::{
        io::{self, Write as _},
        net::TcpStream,
    },
};

pub struct HttpResponse {
    version: Version,
    status: StatusCode,
    headers: ArrayMap<HeaderName, String, 64>,
    body: Body,
}

impl HttpResponse {
    #[allow(clippy::new_ret_no_self)]
    pub const fn new(status: StatusCode) -> HttpResponseBuilder {
        HttpResponseBuilder {
            inner: Self {
                version: Version::Http10,
                status,
                headers: ArrayMap::new(),
                body: Body::Empty,
            },
        }
    }

    pub const fn ok() -> HttpResponseBuilder {
        Self::new(StatusCode::OK)
    }

    pub const fn not_found() -> HttpResponseBuilder {
        Self::new(StatusCode::NOT_FOUND)
    }

    pub const fn internal_server_error() -> HttpResponseBuilder {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub const fn bad_request() -> HttpResponseBuilder {
        Self::new(StatusCode::BAD_REQUEST)
    }

    pub const fn version(&self) -> Version {
        self.version
    }

    pub const fn status(&self) -> StatusCode {
        self.status
    }

    pub(crate) fn into_stream(self, compress: bool, stream: &mut TcpStream) -> io::Result<()> {
        write!(
            stream,
            "{} {} {}\r\n",
            self.version,
            self.status.0,
            self.status.phrase()
        )?;

        for (key, value) in &self.headers {
            write!(stream, "{}: {}\r\n", key, value)?;
        }

        fn write_bytes(
            headers: &ArrayMap<HeaderName, String, 64>,
            bytes: &[u8],
            compress: bool,
            stream: &mut TcpStream,
        ) -> io::Result<()> {
            let pre_compressed = {
                match headers.get(&CONTENT_ENCODING) {
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

        match self.body {
            Body::None | Body::Empty => {
                write!(stream, "Content-Length: 0\r\n")?;
            }
            Body::Bytes(bytes) => {
                write_bytes(&self.headers, bytes, compress, stream)?;
            }
            Body::Vector(bytes) => {
                write_bytes(&self.headers, bytes.as_slice(), compress, stream)?;
            }
        }

        Ok(())
    }
}

pub struct HttpResponseBuilder {
    inner: HttpResponse,
}

impl HttpResponseBuilder {
    pub const fn status(mut self, status: StatusCode) -> Self {
        self.inner.status = status;

        self
    }

    pub fn header<V>(mut self, key: HeaderName, value: V) -> Self
    where
        V: ToString,
    {
        self.inner.headers.insert(key, value.to_string());

        self
    }

    pub fn body<B>(mut self, body: B) -> HttpResponse
    where
        B: Into<Body>,
    {
        self.inner.body = body.into();

        self.inner
    }

    pub fn finish(self) -> HttpResponse {
        self.inner
    }
}
