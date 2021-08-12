use {
    opal::Template,
    std::io::{self, Write},
};

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
            percent_encoding::percent_encode(self.0.as_ref(), percent_encoding::CONTROLS)
        )?;

        Ok(())
    }

    fn size_hint(&self) -> usize {
        let len = self.0.as_ref().len();

        len + (len / 2)
    }
}