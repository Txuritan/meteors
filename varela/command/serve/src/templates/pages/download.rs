#[derive(opal::Template)]
#[template(path = "pages/download.hbs")]
pub struct Download {}

impl Download {
    pub fn new() -> Self {
        Self {}
    }
}
