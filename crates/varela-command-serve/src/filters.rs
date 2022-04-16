pub fn percent_encode<B: AsRef<[u8]>>(bytes: B) -> PercentEncode<B> {
    PercentEncode(bytes)
}

pub struct PercentEncode<B>(B);

impl<B: AsRef<[u8]>> opal::Template for PercentEncode<B> {
    fn render<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where
        W: opal::io::Write,
    {
        struct Wrapper<'a>(enrgy::http::encoding::percent::PercentEncode<'a>);

        impl<'a> opal::io::Display for Wrapper<'a> {
            fn fmt<W>(&self, formatter: &mut opal::io::Formatter<'_, W>) -> Result<(), W::Error>
            where
                W: opal::io::Write + ?Sized,
            {
                for c in (self.0).clone() {
                    formatter.write_str(c)?
                }

                Ok(())
            }
        }

        let encoded = enrgy::http::encoding::percent::percent_encode(
            self.0.as_ref(),
            enrgy::http::encoding::percent::CONTROLS,
        );

        opal::io::write!(writer, "{}", Wrapper(encoded))?;

        Ok(())
    }

    fn size_hint(&self) -> usize {
        let len = self.0.as_ref().len();

        len + (len / 2)
    }
}
