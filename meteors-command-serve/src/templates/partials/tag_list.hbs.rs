impl ::opal::Template for TagList {
#[allow(dead_code, unused_variables, clippy::if_same_then_else)]
    fn size_hint(&self) -> usize {
        let mut hint = 0;hint += 42;
for (i, (kind, tag)) in self.tags.iter().enumerate() {
hint += 21;
hint +=  kind.class() .len();
hint += 23;
hint += &crate::filters::percent_encode(&tag.text).size_hint();
hint += 2;
hint +=  &tag.text .len();
hint += 11;
if i != (&self.tags.len() - 1) {
hint += 2;
}
}
hint += 15;
        hint    }
#[allow(unused_imports)]
    fn render<W>(&self, writer: &mut W) -> ::std::io::Result<()>
        where
            W: ::std::io::Write,
        {
use {::opal::Template as _, std::io::Write as _};
write!(writer, "<div class=\"tags\">\r\n    <span role=\"list\">")?;
for (i, (kind, tag)) in self.tags.iter().enumerate() {
write!(writer, "<wbr /><a class=\"tag ")?;
write!(writer, "{}",  kind.class() )?;
write!(writer, "\" href=\"/search?search=")?;
 crate::filters::percent_encode(&tag.text).render(writer) ?;
write!(writer, "\">")?;
write!(writer, "{}",  &tag.text )?;
write!(writer, "</a><wbr />")?;
if i != (&self.tags.len() - 1) {
write!(writer, ", ")?;
}
}
write!(writer, "</span>\r\n</div>")?;
        Ok(())
    }
}
