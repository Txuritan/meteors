use {
    crate::{
        extractor::Extractor,
        handler::{Handler, HandlerService},
        http::Method,
        service::BoxedService,
        Error, HttpRequest, HttpResponse, Responder,
    },
    std::marker::PhantomData,
};

pub fn to<F, T, R>(handler: F) -> Route<'static, RT>
where
    F: Handler<T, R> + Send + Sync + 'static,
    T: Extractor<Error = Error> + Send + Sync + 'static,
    R: Responder + Send + Sync + 'static,
{
    Route {
        method: None,
        path: None,
        service: BoxedService::new(HandlerService::new(handler)),
        _t: PhantomData,
    }
}

macro_rules! route {
    ($($fn:ident[$method:expr],)*) => {
        $(
            pub fn $fn(path: &str) -> Route<'_, RM> {
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

pub struct RM;

pub struct RT;

pub struct Route<'s, K> {
    pub(crate) method: Option<Method>,
    pub(crate) path: Option<&'s str>,
    pub(crate) service: BoxedService<HttpRequest, HttpResponse, Error>,
    _t: PhantomData<K>,
}

impl<'s, K> Route<'s, K> {
    #[inline]
    pub(crate) fn new(method: Option<Method>, path: Option<&'s str>) -> Self {
        Self {
            method,
            path,
            service: BoxedService::new(HandlerService::new(not_found)),
            _t: PhantomData,
        }
    }
}

impl<'s> Route<'s, RM> {
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
