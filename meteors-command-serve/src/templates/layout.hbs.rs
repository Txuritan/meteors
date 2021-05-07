impl<B: Template> ::opal::Template for Layout<B> {
#[allow(dead_code, unused_variables, clippy::if_same_then_else)]
    fn size_hint(&self) -> usize {
        let mut hint = 0;hint += 30;
hint +=  self.width.as_class() .len();
hint += 1;
hint +=  self.theme.as_class() .len();
hint += 129;
hint +=  self.title .len();
hint += 143;
hint +=  self.query .len();
hint += 327;
hint += &self.body.size_hint();
hint += 38;
        hint    }
#[allow(unused_imports)]
    fn render<W>(&self, writer: &mut W) -> ::std::io::Result<()>
        where
            W: ::std::io::Write,
        {
use {::opal::Template as _, std::io::Write as _};
write!(writer, "<!DOCTYPE html>\r\n<html class=\"")?;
write!(writer, "{}",  self.width.as_class() )?;
write!(writer, " ")?;
write!(writer, "{}",  self.theme.as_class() )?;
write!(writer, "\">\r\n\r\n<head>\r\n    <meta charset=\"UTF-8\">\r\n    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\r\n    <title>")?;
write!(writer, "{}",  self.title )?;
write!(writer, " | local archive</title>\r\n    <link rel=\"stylesheet\" href=\"/style.css\">\r\n</head>\r\n\r\n<body>\r\n    <header>\r\n        <nav>\r\n            <a href=\"/")?;
write!(writer, "{}",  self.query )?;
write!(writer, "\" class=\"brand\">local archive</a>\r\n            <div class=\"spacer\"></div>\r\n            <form method=\"get\" action=\"/search\">\r\n                <input id=\"search\" name=\"search\" type=\"text\" placeholder=\"search\" value=\"\" aria-label=\"Search\" />\r\n            </form>\r\n        </nav>\r\n    </header>\r\n    <hr />\r\n    <section>\r\n        ")?;
 self.body.render(writer) ?;
write!(writer, "\r\n    </section>\r\n</body>\r\n\r\n</html>\r\n")?;
        Ok(())
    }
}
