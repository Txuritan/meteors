#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Method {
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

impl From<tiny_http::Method> for Method {
    fn from(method: tiny_http::Method) -> Self {
        Method::from(&method)
    }
}

impl From<&tiny_http::Method> for Method {
    fn from(method: &tiny_http::Method) -> Self {
        match method {
            tiny_http::Method::Get | tiny_http::Method::NonStandard(_) => Method::Get,
            tiny_http::Method::Head => Method::Head,
            tiny_http::Method::Post => Method::Post,
            tiny_http::Method::Put => Method::Put,
            tiny_http::Method::Delete => Method::Delete,
            tiny_http::Method::Connect => Method::Connect,
            tiny_http::Method::Options => Method::Options,
            tiny_http::Method::Trace => Method::Trace,
            tiny_http::Method::Patch => Method::Patch,
        }
    }
}
