use std::ops::{Deref, DerefMut};

use crate::{extractor::Extractor, http::HttpRequest, Error};

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
    type Error = Error;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(Body {
            value: req.body.clone(),
        })
    }
}
