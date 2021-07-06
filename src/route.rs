use crate::{
    extractor::Extractor,
    handler::{Handler, HandlerService},
    http::Method,
    service::BoxedService,
    Error, HttpRequest, HttpResponse, Responder,
};

pub fn to<F, T, R>(handler: F) -> Route<'static>
where
    F: Handler<T, R> + Send + Sync + 'static,
    T: Extractor<Error = Error> + Send + Sync + 'static,
    R: Responder + Send + Sync + 'static,
{
    Route::new(None, None).to(handler)
}

macro_rules! route {
    ($($fn:ident[$method:expr],)*) => {
        $(
            pub fn $fn(path: &str) -> Route {
                Route::new(Some($method), Some(path))
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
    pub(crate) method: Option<Method>,
    pub(crate) path: Option<&'s str>,
    pub(crate) service: BoxedService<HttpRequest, HttpResponse, Error>,
}

impl<'s> Route<'s> {
    #[inline]
    pub(crate) fn new(method: Option<Method>, path: Option<&'s str>) -> Self {
        Self {
            method,
            path,
            service: BoxedService::new(HandlerService::new(not_found)),
        }
    }

    pub fn to<F, T, R>(mut self, handler: F) -> Self
    where
        F: Handler<T, R> + Send + Sync + 'static,
        T: Extractor<Error = Error> + Send + Sync + 'static,
        R: Responder + Send + Sync + 'static,
    {
        self.service = BoxedService::new(HandlerService::new(handler));

        self
    }
}
