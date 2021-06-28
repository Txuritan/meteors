impl ::opal::Template for Download {
#[allow(dead_code, unused_variables, clippy::if_same_then_else, clippy::branches_sharing_code)]
    fn size_hint(&self) -> usize {
        let mut hint = 0;hint += 189;
        hint    }
#[allow(unused_imports, clippy::branches_sharing_code)]
    fn render<W>(&self, writer: &mut W) -> ::std::io::Result<()>
        where
            W: ::std::io::Write,
        {
use {::opal::Template as _, std::io::Write as _};
write!(writer, "<main>\r\n    <form method=\"post\" action=\"/download\">\r\n        <input id=\"download\" name=\"download\" type=\"url\" placeholder=\"download\" value=\"\" aria-label=\"Download\" />\r\n    </form>\r\n</main>\r\n")?;
        Ok(())
    }
}
