impl<'s> ::opal::Template for Index<'s> {
#[allow(dead_code, unused_variables, clippy::if_same_then_else)]
    fn size_hint(&self) -> usize {
        let mut hint = 0;hint += 12;
if self.stories.is_empty() {
hint += 6;
} else {
hint += 10;
let len = self.stories.len() -1;
hint += 10;
for (i, story) in self.stories.iter().enumerate() {
hint += 14;
hint += &story.size_hint();
hint += 14;
if i != len {
hint += 38;
}
hint += 10;
}
hint += 6;
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
if self.stories.is_empty() {
write!(writer, "\r\n    ")?;
} else {
write!(writer, "\r\n        ")?;
let len = self.stories.len() -1;
write!(writer, "\r\n        ")?;
for (i, story) in self.stories.iter().enumerate() {
write!(writer, "\r\n            ")?;
story.render(writer)?;
write!(writer, "\r\n            ")?;
if i != len {
write!(writer, "\r\n                <hr />\r\n            ")?;
}
write!(writer, "\r\n        ")?;
}
write!(writer, "\r\n    ")?;
}
write!(writer, "\r\n</main>")?;
        Ok(())
    }
}
