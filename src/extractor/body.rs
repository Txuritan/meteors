use {
    super::{Extractor, ExtractorError},
    crate::HttpRequest,
    std::ops::Deref,
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

impl Extractor for Body {
    type Error = ExtractorError;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(Body {
            value: req.body.clone(),
        })
    }
}
