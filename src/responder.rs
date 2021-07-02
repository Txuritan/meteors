use crate::{HttpRequest, HttpResponse};

pub trait Responder {
    fn respond_to(self, req: &HttpRequest) -> HttpResponse;
}

impl Responder for HttpResponse {
    fn respond_to(self, _req: &HttpRequest) -> HttpResponse {
        self
    }
}
