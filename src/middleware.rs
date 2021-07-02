use crate::{HttpRequest, HttpResponse};

pub trait Middleware {
    fn before(&self, req: &mut HttpRequest);
    fn after(&self, req: &HttpRequest, res: &HttpResponse);
}
