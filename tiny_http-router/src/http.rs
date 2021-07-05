use {
    crate::{router::Extensions, Header, Method, StatusCode},
    std::{
        collections::BTreeMap,
        io::Cursor,
        sync::{mpsc::Receiver, Arc},
    },
    tiny_http::HTTPVersion,
};

pub struct HttpRequest {
    pub(crate) inner: tiny_http::Request,
    pub(crate) data: Arc<Extensions>,
    pub(crate) ext: Extensions,
    pub(crate) parameters: BTreeMap<String, String>,
    pub(crate) query: BTreeMap<String, String>,
    pub(crate) raw_query: String,
}

impl HttpRequest {
    pub fn ext(&self) -> &Extensions {
        &self.ext
    }

    pub fn ext_mut(&mut self) -> &mut Extensions {
        &mut self.ext
    }

    pub fn http_version(&self) -> &HTTPVersion {
        self.inner.http_version()
    }

    pub fn method(&self) -> Method {
        Method::from(self.inner.method())
    }

    pub fn url(&self) -> &str {
        self.inner.url()
    }
}

pub struct HttpResponse {
    pub(crate) inner: tiny_http::Response<Cursor<Vec<u8>>>,
}

impl HttpResponse {
    #[inline]
    pub fn new(
        status_code: StatusCode,
        headers: Vec<Header>,
        data: Cursor<Vec<u8>>,
        data_length: Option<usize>,
        additional_headers: Option<Receiver<Header>>,
    ) -> Self {
        Self {
            inner: tiny_http::Response::new(
                status_code,
                headers,
                data,
                data_length,
                additional_headers,
            ),
        }
    }

    #[inline]
    pub fn from_data<D>(data: D) -> Self
    where
        D: Into<Vec<u8>>,
    {
        Self {
            inner: tiny_http::Response::from_data(data),
        }
    }

    #[inline]
    pub fn from_string<S>(data: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            inner: tiny_http::Response::from_string(data),
        }
    }

    #[inline]
    pub fn with_header<H>(mut self, header: H) -> Self
    where
        H: Into<Header>,
    {
        self.inner = self.inner.with_header(header);

        self
    }

    #[inline]
    pub fn with_status_code<S>(mut self, code: S) -> Self
    where
        S: Into<StatusCode>,
    {
        self.inner = self.inner.with_status_code(code);

        self
    }

    pub fn status_code(&self) -> StatusCode {
        self.inner.status_code()
    }
}
