pub static NAV: Nav = Nav {
    entries: [NavEntry {
        url: "/fandoms",
        name: "fandoms",
    }],
};

#[derive(Clone, Copy, opal::Template)]
#[template(path = "partials/nav.hbs")]
pub struct Nav {
    entries: [NavEntry; 1],
}

#[derive(Clone, Copy, opal::Template)]
#[template(path = "partials/nav-entry.hbs")]
pub struct NavEntry {
    url: &'static str,
    name: &'static str,
}
