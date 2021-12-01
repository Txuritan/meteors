pub trait StringExt {
    fn trim(&mut self);

    fn trim_matches(&mut self, rem: &str);

    fn trim_start(&mut self);

    fn trim_start_matches(&mut self, rem: &str);

    fn trim_end(&mut self);

    fn trim_end_matches(&mut self, rem: &str);
}

impl StringExt for String {
    fn trim(&mut self) {
        self.trim_start();
        self.trim_end();
    }

    fn trim_matches(&mut self, rem: &str) {
        self.trim_start_matches(rem);
        self.trim_end_matches(rem);
    }

    fn trim_start(&mut self) {
        while self.starts_with(char::is_whitespace) {
            self.drain(..1);
        }
    }

    fn trim_start_matches(&mut self, rem: &str) {
        while self.starts_with(rem) {
            self.drain(..rem.len());
        }
    }

    fn trim_end(&mut self) {
        while self.ends_with(char::is_whitespace) {
            self.truncate(self.len().saturating_sub(1));
        }
    }

    fn trim_end_matches(&mut self, rem: &str) {
        while self.ends_with(rem) {
            self.truncate(self.len().saturating_sub(rem.len()));
        }
    }
}
