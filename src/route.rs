use crate::{
    extractor::{Extractor, ExtractorError},
    handler::{Handler, HandlerError, HandlerService},
    service::BoxedService,
    HttpRequest, HttpResponse, Method, Responder,
};

macro_rules! route {
    ($($fn:ident[$method:expr],)*) => {
        $(
            pub fn $fn(path: &str) -> Route {
                Route::new($method, path)
            }
        )*
    };
}

route![
    get[Method::Get],
    head[Method::Head],
    post[Method::Post],
    put[Method::Put],
    delete[Method::Delete],
    connect[Method::Connect],
    options[Method::Options],
    trace[Method::Trace],
    patch[Method::Patch],
];

pub(crate) fn not_found() -> HttpResponse {
    HttpResponse::not_found().finish()
}

pub struct Route<'s> {
    pub(crate) method: Method,
    pub(crate) path: &'s str,
    pub(crate) service: BoxedService<HttpRequest, HttpResponse, HandlerError>,
}

impl<'s> Route<'s> {
    #[inline]
    pub(crate) fn new(method: Method, path: &'s str) -> Self {
        Self {
            method,
            path,
            service: BoxedService::new(HandlerService::new(not_found)),
        }
    }

    pub fn to<F, T, R>(mut self, handler: F) -> Self
    where
        F: Handler<T, R> + Send + Sync + 'static,
        T: Extractor<Error = ExtractorError> + Send + Sync + 'static,
        R: Responder + Send + Sync + 'static,
    {
        self.service = BoxedService::new(HandlerService::new(handler));

        self
    }
}
