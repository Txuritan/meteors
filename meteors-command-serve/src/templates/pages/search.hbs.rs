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
hint += 279;
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
hint += 122;
for (entry, count) in self.stats.warnings.iter() {
hint += 22;
 let encoded = crate::filters::percent_encode(&entry.text); 
hint += 81;
hint += &encoded.size_hint();
hint += 40;
hint += &encoded.size_hint();
hint += 2;
hint +=  entry.text .len();
hint += 2;
hint += 56;
}
hint += 124;
for (entry, count) in self.stats.categories.iter() {
hint += 22;
 let encoded = crate::filters::percent_encode(&entry.text); 
hint += 81;
hint += &encoded.size_hint();
hint += 40;
hint += &encoded.size_hint();
hint += 2;
hint +=  entry.text .len();
hint += 2;
hint += 56;
}
hint += 121;
for (entry, count) in self.stats.origins.iter() {
hint += 22;
 let encoded = crate::filters::percent_encode(&entry.text); 
hint += 81;
hint += &encoded.size_hint();
hint += 40;
hint += &encoded.size_hint();
hint += 2;
hint +=  entry.text .len();
hint += 2;
hint += 56;
}
hint += 204;
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
 story.render(writer) ?;
write!(writer, "\r\n            ")?;
if i != len {
write!(writer, "\r\n                <hr />\r\n            ")?;
}
write!(writer, "\r\n        ")?;
}
write!(writer, "\r\n    ")?;
}
write!(writer, "\r\n</main>\r\n\r\n<aside>\r\n    <form action=\"/\" method=\"get\" id=\"filter\">\r\n        <button type=\"submit\">Sort and Filter</button>\r\n        <fieldset>\r\n            <legend>Include</legend>\r\n            <details open=\"open\">\r\n                <summary>Ratings</summary>\r\n                ")?;
for (rating, count) in self.stats.ratings.iter() {
write!(writer, "\r\n                    ")?;
 let encoded = crate::filters::percent_encode(rating.class()); 
write!(writer, "\r\n                    <span>\r\n                        <input type=\"radio\" name=\"rating\" id=\"")?;
 encoded.render(writer) ?;
write!(writer, "\">\r\n                        <label for=\"")?;
 encoded.render(writer) ?;
write!(writer, "\">")?;
write!(writer, "{}",  rating.name() )?;
write!(writer, " (")?;
write!(writer, "{}",  count )?;
write!(writer, ")</label>\r\n                    </span>\r\n                ")?;
}
write!(writer, "\r\n            </details>\r\n            <details open=\"open\">\r\n                <summary>Warnings</summary>\r\n                ")?;
for (entry, count) in self.stats.warnings.iter() {
write!(writer, "\r\n                    ")?;
 let encoded = crate::filters::percent_encode(&entry.text); 
write!(writer, "\r\n                    <span>\r\n                        <input type=\"checkbox\" id=\"")?;
 encoded.render(writer) ?;
write!(writer, "\">\r\n                        <label for=\"")?;
 encoded.render(writer) ?;
write!(writer, "\">")?;
write!(writer, "{}",  entry.text )?;
write!(writer, " (")?;
write!(writer, "{}",  count )?;
write!(writer, ")</label>\r\n                    </span>\r\n                ")?;
}
write!(writer, "\r\n            </details>\r\n            <details open=\"open\">\r\n                <summary>Categories</summary>\r\n                ")?;
for (entry, count) in self.stats.categories.iter() {
write!(writer, "\r\n                    ")?;
 let encoded = crate::filters::percent_encode(&entry.text); 
write!(writer, "\r\n                    <span>\r\n                        <input type=\"checkbox\" id=\"")?;
 encoded.render(writer) ?;
write!(writer, "\">\r\n                        <label for=\"")?;
 encoded.render(writer) ?;
write!(writer, "\">")?;
write!(writer, "{}",  entry.text )?;
write!(writer, " (")?;
write!(writer, "{}",  count )?;
write!(writer, ")</label>\r\n                    </span>\r\n                ")?;
}
write!(writer, "\r\n            </details>\r\n            <details open=\"open\">\r\n                <summary>Origins</summary>\r\n                ")?;
for (entry, count) in self.stats.origins.iter() {
write!(writer, "\r\n                    ")?;
 let encoded = crate::filters::percent_encode(&entry.text); 
write!(writer, "\r\n                    <span>\r\n                        <input type=\"checkbox\" id=\"")?;
 encoded.render(writer) ?;
write!(writer, "\">\r\n                        <label for=\"")?;
 encoded.render(writer) ?;
write!(writer, "\">")?;
write!(writer, "{}",  entry.text )?;
write!(writer, " (")?;
write!(writer, "{}",  count )?;
write!(writer, ")</label>\r\n                    </span>\r\n                ")?;
}
write!(writer, "\r\n            </details>\r\n        </fieldset>\r\n        <fieldset>\r\n            <legend>Exclude</legend>\r\n        </fieldset>\r\n        <button type=\"submit\"> Sort and Filter</button>\r\n    </form>\r\n</aside>")?;
        Ok(())
    }
}
