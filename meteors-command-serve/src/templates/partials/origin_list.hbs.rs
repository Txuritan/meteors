impl ::opal::Template for OriginList {
#[allow(dead_code, clippy::if_same_then_else)]
    fn size_hint(&self) -> usize {
        let mut hint = 0;hint += 45;
for (i, origin) in self.origins.iter().enumerate() {
hint += 32;
hint += &crate::filters::percent_encode(&origin.text).size_hint();
hint += 2;
hint +=  &origin.text .len();
hint += 11;
if i != (self.origins.len() - 1) {
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
write!(writer, "<div class=\"origins\">\r\n    <span role=\"list\">")?;
for (i, origin) in self.origins.iter().enumerate() {
write!(writer, "<wbr /><a  href=\"/search?search=")?;
 crate::filters::percent_encode(&origin.text).render(writer) ?;
write!(writer, "\">")?;
write!(writer, "{}",  &origin.text )?;
write!(writer, "</a><wbr />")?;
if i != (self.origins.len() - 1) {
write!(writer, ", ")?;
}
}
write!(writer, "</span>\r\n</div>")?;
        Ok(())
    }
}
