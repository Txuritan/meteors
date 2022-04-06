# WIP Features

## Routing

```rust
enrgy::router! {
    GET handlers::index

    "download" {
        GET handlers::download_get
        POST handlers::download_post
    }
    "story" {
        < id: Id > {
            < chapter: usize > {
                GET handlers::story
            }
        }
    }

    ( "author" | "origin" | "tag" ) {
        < id: Id > {
            GET handlers::entity
        }
    }

    "search" { GET handlers::search }
    "search2" { GET handlers::search_v2 }

    "opds" { GET handlers::catalog }

    "style.css" { GET handlers::style }
    "favicon.ico" { GET handlers::favicon }
};
```
