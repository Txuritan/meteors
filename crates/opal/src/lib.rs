use ::std::convert::Infallible;

pub use opal_macros::Template;

pub trait Template {
    fn size_hint(&self) -> usize;

    fn render<W>(&self, writer: &mut W) -> Result<(), W::Error>
    where
        W: crate::io::Write;

    fn render_as_string(&self) -> Result<String, Infallible>
    where
        Self: Sized,
    {
        let mut buf = String::with_capacity(self.size_hint());

        self.render(&mut buf)?;

        // SAFETY: The buffer is built using `write` calls, and everything is already a Rust string
        // Ok(unsafe { String::from_utf8_unchecked(buf) })

        Ok(buf)
    }

    fn render_into_string(self) -> Result<String, Infallible>
    where
        Self: Sized,
    {
        self.render_as_string()
    }
}

#[doc(hidden)]
pub mod io {
    //! Interior IO util, use at own risk.

    pub use vfmt::{self, uDisplay as Display, uWrite as Write, uwrite as write, Formatter};
}
