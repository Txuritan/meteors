use {
    super::{Extractor, ExtractorError},
    crate::HttpRequest,
    std::ops::Deref,
};

pub struct OptionalHeader<const KEY: &'static str> {
    value: Option<String>,
}

impl<const KEY: &'static str> Deref for OptionalHeader<KEY> {
    type Target = Option<String>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<const KEY: &'static str> Extractor for OptionalHeader<KEY> {
    type Error = ExtractorError;

    fn extract(req: &mut HttpRequest) -> Result<Self, Self::Error> {
        Ok(Self {
            value: req
                .header_data
                .headers
                .iter()
                .find(|(key, _)| key.eq_ignore_ascii_case(KEY))
                .map(|(_, value)| value.to_string()),
        })
    }
}
