use std::ops::{Deref, DerefMut};

use crate::{
    extractor::Extractor,
    http::{HttpRequest, HttpResponse},
    response::IntoResponse,
};

pub struct Body {
    value: Vec<u8>,
}

impl const Deref for Body {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl const DerefMut for Body {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl Extractor for Body {
    type Error = BodyRejection;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        let value = match req.body.as_ref() {
            Some(body) => body.as_vec(),
            None => return Err(BodyRejection {}),
        };

        Ok(Body { value })
    }
}

pub struct BodyRejection {}

impl IntoResponse for BodyRejection {
    fn into_response(self) -> HttpResponse {
        todo!()
    }
}
