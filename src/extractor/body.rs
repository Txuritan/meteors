use {
    crate::{extractor::Extractor, Error, HttpRequest},
    std::ops::{Deref, DerefMut},
};

pub struct Body {
    value: Vec<u8>,
}

impl Deref for Body {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl DerefMut for Body {
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
