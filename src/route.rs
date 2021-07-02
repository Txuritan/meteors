use crate::{
    extractor::{Extractor, ExtractorError},
    handler::Handler,
    handler::HandlerService,
    service::BoxedService,
    Error, HttpRequest, HttpResponse, Method,
};

pub fn get(path: &str) -> Route {
    Route::new(Method::Get, path)
}

pub fn post(path: &str) -> Route {
    Route::new(Method::Post, path)
}

pub struct Route<'s> {
    pub(crate) method: Method,
    pub(crate) path: &'s str,
    pub(crate) service: BoxedService<HttpRequest, HttpResponse, Error>,
}

impl<'s> Route<'s> {
    #[inline]
    pub(crate) fn new(method: Method, path: &'s str) -> Self {
        Self {
            method,
            path,
            service: BoxedService::new(HandlerService::new(Self::not_found)),
        }
    }

    pub(crate) fn not_found() -> Result<HttpResponse, Error> {
        todo!()
    }

    pub fn to<F, T>(mut self, handler: F) -> Self
    where
        F: Handler<T> + Send + Sync + 'static,
        T: Extractor<Error = ExtractorError> + Send + Sync + 'static,
    {
        self.service = BoxedService::new(HandlerService::new(handler));

        self
    }
}
