use crate::{
    extractor::Extractor,
    handler::{Handler, HandlerService},
    http::{HttpMethod, HttpRequest, HttpResponse},
    response::IntoResponse,
    service::BoxedService,
};

pub fn to<F, T, E>(handler: F) -> Route<'static>
where
    F: Handler<T> + Send + Sync + 'static,
    T: Extractor<Error = E> + Send + Sync + 'static,
    E: IntoResponse + Send + Sync + 'static,
{
    Route {
        method: HttpMethod::Get,
        path: "/<to>",
        service: BoxedService::new(HandlerService::new(handler)),
    }
}

macro_rules! route {
    ($($fn:ident[$method:expr],)*) => {
        $(
            pub fn $fn(path: &str) -> Route<'_> {
                Route::new($method, path)
            }
        )*
    };
}

route![
    get[HttpMethod::Get],
    head[HttpMethod::Head],
    post[HttpMethod::Post],
    put[HttpMethod::Put],
    delete[HttpMethod::Delete],
    connect[HttpMethod::Connect],
    options[HttpMethod::Options],
    trace[HttpMethod::Trace],
    patch[HttpMethod::Patch],
];

pub(crate) fn not_found() -> HttpResponse {
    HttpResponse::not_found()
}

pub struct Route<'s> {
    pub(crate) method: HttpMethod,
    pub(crate) path: &'s str,
    pub(crate) service: BoxedService<HttpRequest, HttpResponse, HttpResponse>,
}

impl<'s> Route<'s> {
    #[inline]
    pub(crate) fn new(method: HttpMethod, path: &'s str) -> Self {
        Self {
            method,
            path,
            service: BoxedService::new(HandlerService::new(not_found)),
        }
    }

    pub fn to<F, T, E>(mut self, handler: F) -> Self
    where
        F: Handler<T> + Send + Sync + 'static,
        T: Extractor<Error = E> + Send + Sync + 'static,
        E: IntoResponse + Send + Sync + 'static,
    {
        self.service = BoxedService::new(HandlerService::new(handler));

        self
    }
}
