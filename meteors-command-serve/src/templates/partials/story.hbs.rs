impl<'s> ::opal::Template for StoryCard<'s> {
#[allow(dead_code, unused_variables, clippy::if_same_then_else)]
    fn size_hint(&self) -> usize {
        let mut hint = 0;hint += 99;
hint +=  self.rating.class() .len();
hint += 2;
hint +=  self.rating.symbol() .len();
hint += 24;
hint +=  self.id .len();
hint += 2;
hint +=  self.query .len();
hint += 16;
hint +=  self.info.title .len();
hint += 21;
for (i, author) in self.authors.iter().enumerate() {
hint += 42;
hint += &crate::filters::percent_encode(&author.text).size_hint();
hint += 2;
hint +=  author.text .len();
hint += 4;
if i != (self.authors.len() - 1) {
hint += 1;
}
hint += 14;
}
hint += 57;
for (i, category) in self.categories.iter().enumerate() {
hint += 24;
hint +=  category.text .len();
hint += 7;
if i != (self.categories.len() - 1) {
hint += 1;
}
hint += 14;
}
hint += 35;
hint += &self.origins.size_hint();
hint += 6;
hint += &self.tags.size_hint();
hint += 27;
hint +=  self.info.summary .len();
hint += 84;
hint += 23;
        hint    }
#[allow(unused_imports)]
    fn render<W>(&self, writer: &mut W) -> ::std::io::Result<()>
        where
            W: ::std::io::Write,
        {
use {::opal::Template as _, std::io::Write as _};
write!(writer, "<article class=\"story\">\r\n    <header>\r\n        <div class=\"left\">\r\n            <span class=\"rating ")?;
write!(writer, "{}",  self.rating.class() )?;
write!(writer, "\">")?;
write!(writer, "{}",  self.rating.symbol() )?;
write!(writer, "</span> <a href=\"/story/")?;
write!(writer, "{}",  self.id )?;
write!(writer, "/1")?;
write!(writer, "{}",  self.query )?;
write!(writer, "\" class=\"title\">")?;
write!(writer, "{}",  self.info.title )?;
write!(writer, "</a> by\r\n            ")?;
for (i, author) in self.authors.iter().enumerate() {
write!(writer, "\r\n                <a href=\"/search?search=")?;
 crate::filters::percent_encode(&author.text).render(writer) ?;
write!(writer, "\">")?;
write!(writer, "{}",  author.text )?;
write!(writer, "</a>")?;
if i != (self.authors.len() - 1) {
write!(writer, ",")?;
}
write!(writer, "\r\n            ")?;
}
write!(writer, "\r\n        </div>\r\n        <p class=\"right\">\r\n            ")?;
for (i, category) in self.categories.iter().enumerate() {
write!(writer, "\r\n                <span>")?;
write!(writer, "{}",  category.text )?;
write!(writer, "</span>")?;
if i != (self.categories.len() - 1) {
write!(writer, ",")?;
}
write!(writer, "\r\n            ")?;
}
write!(writer, "\r\n        </p>\r\n    </header>\r\n    ")?;
 self.origins.render(writer) ?;
write!(writer, "\r\n    ")?;
 self.tags.render(writer) ?;
write!(writer, "\r\n    <div class=\"summary\">")?;
write!(writer, "{}",  self.info.summary )?;
write!(writer, "</div>\r\n    <p class=\"meta\"><span class=\"info\">chapters</span>: <span class=\"count\">")?;
write!(writer, "{}",  self.len )?;
write!(writer, "</span></p>\r\n</article>")?;
        Ok(())
    }
}
