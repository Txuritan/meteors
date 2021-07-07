use std::io::{Result, Write};

pub use opal_macros::Template;

pub trait Template {
    fn size_hint(&self) -> usize;

    fn render<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write;

    fn render_as_string(&self) -> Result<String>
    where
        Self: Sized,
    {
        let mut buf = Vec::with_capacity(self.size_hint());

        self.render(&mut buf)?;

        // SAFETY: The buffer is built using `write` calls, and everything is already a Rust string
        Ok(unsafe { String::from_utf8_unchecked(buf) })
    }

    fn render_into_string(self) -> Result<String>
    where
        Self: Sized,
    {
        self.render_as_string()
    }
}
