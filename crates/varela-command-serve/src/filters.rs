use std::io::{self, Write};

use opal::Template;

pub fn percent_encode<B: AsRef<[u8]>>(bytes: B) -> PercentEncode<B> {
    PercentEncode(bytes)
}

pub struct PercentEncode<B>(B);

impl<B: AsRef<[u8]>> Template for PercentEncode<B> {
    fn render<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: Write,
    {
        write!(
            writer,
            "{}",
            enrgy::http::encoding::percent::percent_encode(self.0.as_ref(), enrgy::http::encoding::percent::CONTROLS)
        )?;

        Ok(())
    }

    fn size_hint(&self) -> usize {
        let len = self.0.as_ref().len();

        len + (len / 2)
    }
}
