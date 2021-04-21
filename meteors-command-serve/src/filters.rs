use sailfish::runtime::{Buffer, Render, RenderError};

pub fn percent_encode<B: AsRef<[u8]>>(bytes: B) -> PercentEncode<B> {
    PercentEncode(bytes)
}

pub struct PercentEncode<B>(B);

impl<B: AsRef<[u8]>> Render for PercentEncode<B> {
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        use std::fmt::Write;

        write!(b, "{}", percent_encoding::percent_encode(self.0.as_ref(), &percent_encoding::CONTROLS)).map_err(RenderError::from)
    }
}