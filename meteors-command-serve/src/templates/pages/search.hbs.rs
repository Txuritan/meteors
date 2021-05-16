impl<'s> ::opal::Template for Search<'s> {
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
hint += 241;
for (rating, count) in self.stats.ratings.iter() {
hint += 22;
let encoded = crate::filters::percent_encode(rating.class());
hint += 92;
hint += &encoded.size_hint();
hint += 40;
hint += &encoded.size_hint();
hint += 2;
hint +=  rating.name() .len();
hint += 2;
hint += 56;
}
hint += 38;
let lists = vec![
                ("Categories", &self.stats.categories),
                ("Origins", &self.stats.origins),
                ("Warnings", &self.stats.warnings),
                ("Pairings", &self.stats.pairings),
                ("Characters", &self.stats.characters),
            ];
hint += 14;
for (name, list) in lists {
hint += 70;
hint +=  name .len();
hint += 32;
for (entry, count) in list.iter() {
hint += 26;
let encoded = crate::filters::percent_encode(&entry.text);
hint += 89;
hint += &encoded.size_hint();
hint += 44;
hint += &encoded.size_hint();
hint += 2;
hint +=  entry.text .len();
hint += 2;
hint += 64;
}
hint += 42;
}
hint += 101;
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
write!(writer, "\r\n</main>\r\n\r\n<aside>\r\n    <form action=\"/\" method=\"get\" id=\"filter\">\r\n        <button type=\"submit\">Sort and Filter</button>\r\n        <fieldset>\r\n            <details open=\"open\">\r\n                <summary>Ratings</summary>\r\n                ")?;
for (rating, count) in self.stats.ratings.iter() {
write!(writer, "\r\n                    ")?;
let encoded = crate::filters::percent_encode(rating.class());
write!(writer, "\r\n                    <span>\r\n                        <input type=\"radio\" name=\"rating\" id=\"")?;
encoded.render(writer)?;
write!(writer, "\">\r\n                        <label for=\"")?;
encoded.render(writer)?;
write!(writer, "\">")?;
write!(writer, "{}",  rating.name() )?;
write!(writer, " (")?;
write!(writer, "{}",  count )?;
write!(writer, ")</label>\r\n                    </span>\r\n                ")?;
}
write!(writer, "\r\n            </details>\r\n            ")?;
let lists = vec![
                ("Categories", &self.stats.categories),
                ("Origins", &self.stats.origins),
                ("Warnings", &self.stats.warnings),
                ("Pairings", &self.stats.pairings),
                ("Characters", &self.stats.characters),
            ];
write!(writer, "\r\n            ")?;
for (name, list) in lists {
write!(writer, "\r\n                <details open=\"open\">\r\n                    <summary>")?;
write!(writer, "{}",  name )?;
write!(writer, "</summary>\r\n                    ")?;
for (entry, count) in list.iter() {
write!(writer, "\r\n                        ")?;
let encoded = crate::filters::percent_encode(&entry.text);
write!(writer, "\r\n                        <span>\r\n                            <input type=\"checkbox\" id=\"")?;
encoded.render(writer)?;
write!(writer, "\">\r\n                            <label for=\"")?;
encoded.render(writer)?;
write!(writer, "\">")?;
write!(writer, "{}",  entry.text )?;
write!(writer, " (")?;
write!(writer, "{}",  count )?;
write!(writer, ")</label>\r\n                        </span>\r\n                    ")?;
}
write!(writer, "\r\n                </details>\r\n            ")?;
}
write!(writer, "\r\n        </fieldset>\r\n        <button type=\"submit\"> Sort and Filter</button>\r\n    </form>\r\n</aside>")?;
        Ok(())
    }
}
