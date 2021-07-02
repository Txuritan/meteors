use {
    super::{Body, Version},
    crate::StatusCode,
    std::collections::BTreeMap,
};

pub struct HttpResponse {
    version: Version,
    status: StatusCode,
    headers: BTreeMap<&'static str, String>,
    body: Body,
}

impl HttpResponse {
    pub fn new(status: StatusCode) -> Self {
        Self {
            version: Version::Http10,
            status,
            headers: BTreeMap::new(),
            body: Body::Empty,
        }
    }

    pub fn ok() -> Self {
        Self::new(StatusCode::OK)
    }

    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND)
    }

    pub fn internal_server_error() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn bad_request() -> Self {
        Self::new(StatusCode::BAD_REQUEST)
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;

        self
    }

    pub fn header<V>(mut self, key: &'static str, value: V) -> Self
    where
        V: ToString,
    {
        self.headers.insert(key, value.to_string());

        self
    }

    pub fn body<B>(mut self, body: B) -> Self
    where
        B: Into<Body>,
    {
        self.body = body.into();

        self
    }

    pub(crate) fn into_bytes(self) -> Vec<u8> {
        vec![]
    }
}
