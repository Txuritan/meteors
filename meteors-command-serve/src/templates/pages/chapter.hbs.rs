impl<'s> ::opal::Template for Chapter<'s> {
#[allow(dead_code, unused_variables, clippy::if_same_then_else)]
    fn size_hint(&self) -> usize {
        let mut hint = 0;hint += 12;
let len = self.card.len;
hint += 6;
let id = self.card.id;
hint += 6;
hint += &self.card.size_hint();
hint += 18;
hint +=  self.chapter .len();
hint += 18;
if len != 1 {
hint += 71;
hint +=  id  .len();
hint += 2;
hint +=  self.query .len();
hint += 86;
if self.index.saturating_sub(1) != 0 {
hint += 38;
hint +=  id  .len();
hint += 1;
hint +=  self.query .len();
hint += 41;
} else {
hint += 38;
hint +=  id  .len();
hint += 2;
hint +=  self.query .len();
hint += 41;
}
hint += 18;
if self.index + 1 != len {
hint += 38;
hint +=  id  .len();
hint += 1;
hint +=  self.query .len();
hint += 41;
} else {
hint += 38;
hint +=  id  .len();
hint += 1;
hint +=  self.query .len();
hint += 41;
}
hint += 78;
hint +=  id  .len();
hint += 1;
hint +=  self.query .len();
hint += 68;
}
hint += 9;
        hint    }
#[allow(unused_imports)]
    fn render<W>(&self, writer: &mut W) -> ::std::io::Result<()>
        where
            W: ::std::io::Write,
        {
use {::opal::Template as _, std::io::Write as _};
write!(writer, "<main>\r\n    ")?;
let len = self.card.len;
write!(writer, "\r\n    ")?;
let id = self.card.id;
write!(writer, "\r\n    ")?;
self.card.render(writer)?;
write!(writer, "\r\n    <hr />\r\n    ")?;
write!(writer, "{}",  self.chapter )?;
write!(writer, "\r\n    <hr />\r\n    ")?;
if len != 1 {
write!(writer, "\r\n        <footer>\r\n            <nav>\r\n                <a href=\"/story/")?;
write!(writer, "{}",  id  )?;
write!(writer, "/1")?;
write!(writer, "{}",  self.query )?;
write!(writer, "\" class=\"item\">first</a>\r\n                <div class=\"spacer\"></div>\r\n                ")?;
if self.index.saturating_sub(1) != 0 {
write!(writer, "\r\n                    <a href=\"/story/")?;
write!(writer, "{}",  id  )?;
write!(writer, "/")?;
write!(writer, "{}",  self.index - 1 )?;
write!(writer, "{}",  self.query )?;
write!(writer, "\" class=\"item\">prev</a>\r\n                ")?;
} else {
write!(writer, "\r\n                    <a href=\"/story/")?;
write!(writer, "{}",  id  )?;
write!(writer, "/1")?;
write!(writer, "{}",  self.query )?;
write!(writer, "\" class=\"item\">prev</a>\r\n                ")?;
}
write!(writer, "\r\n                ")?;
if self.index + 1 != len {
write!(writer, "\r\n                    <a href=\"/story/")?;
write!(writer, "{}",  id  )?;
write!(writer, "/")?;
write!(writer, "{}",  self.index + 1 )?;
write!(writer, "{}",  self.query )?;
write!(writer, "\" class=\"item\">next</a>\r\n                ")?;
} else {
write!(writer, "\r\n                    <a href=\"/story/")?;
write!(writer, "{}",  id  )?;
write!(writer, "/")?;
write!(writer, "{}",  len )?;
write!(writer, "{}",  self.query )?;
write!(writer, "\" class=\"item\">next</a>\r\n                ")?;
}
write!(writer, "\r\n                <div class=\"spacer\"></div>\r\n                <a href=\"/story/")?;
write!(writer, "{}",  id  )?;
write!(writer, "/")?;
write!(writer, "{}",  len )?;
write!(writer, "{}",  self.query )?;
write!(writer, "\" class=\"item\">last</a>\r\n            </nav>\r\n        </footer>\r\n    ")?;
}
write!(writer, "\r\n</main>")?;
        Ok(())
    }
}
