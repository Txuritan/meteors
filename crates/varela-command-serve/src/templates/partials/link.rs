#[derive(PartialEq)]
pub enum Contrast {
    High,
    Low,
    Disabled,
}

#[derive(opal::Template)]
#[template(path = "partials/link.hbs")]
pub struct Link {
    contrast: Contrast,
    href: String,
    text: String,
}

impl Link {
    pub fn new(contrast: Contrast, href: String, text: String) -> Self {
        Self {
            contrast,
            href,
            text,
        }
    }
}
