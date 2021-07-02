use crate::{HttpRequest, HttpResponse};

pub trait Responder {
    fn respond_to(self, req: &HttpRequest) -> HttpResponse;
}

impl<T, E> Responder for Result<T, E>
where
    T: Responder,
{
    fn respond_to(self, req: &HttpRequest) -> HttpResponse {
        todo!()
    }
}
