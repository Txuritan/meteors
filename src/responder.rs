use crate::{Error, HttpRequest, HttpResponse};

pub trait Responder {
    fn respond_to(self, req: &HttpRequest) -> Result<HttpResponse, Error>;
}

impl Responder for HttpResponse {
    fn respond_to(self, _req: &HttpRequest) -> Result<HttpResponse, Error> {
        Ok(self)
    }
}

impl<T, E> Responder for Result<T, E>
where
    T: Responder,
    E: Into<Error>,
{
    fn respond_to(self, req: &HttpRequest) -> Result<HttpResponse, Error> {
        match self {
            Ok(res) => res.respond_to(req),
            Err(err) => Err(err.into()),
        }
    }
}

impl Responder for &'static str {
    fn respond_to(self, _req: &HttpRequest) -> Result<HttpResponse, Error> {
        Ok(HttpResponse::ok().body(self))
    }
}

impl Responder for &'static [u8] {
    fn respond_to(self, _req: &HttpRequest) -> Result<HttpResponse, Error> {
        Ok(HttpResponse::ok().body(self))
    }
}

impl Responder for String {
    fn respond_to(self, _req: &HttpRequest) -> Result<HttpResponse, Error> {
        Ok(HttpResponse::ok().body(self))
    }
}

impl Responder for Vec<u8> {
    fn respond_to(self, _req: &HttpRequest) -> Result<HttpResponse, Error> {
        Ok(HttpResponse::ok().body(self))
    }
}
