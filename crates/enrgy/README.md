# enrgy

A (nightly) lightweight synchronous Actix-like HTTP server.

*WARNING*: Do not allow access to this server from the open internet, it has little to no security measures.

## Example

```rust
use enrgy::{web, App, HttpServer, Responder};

fn index() -> impl Responder {
    "Hello World!"
}

fn greet(name: web::Param<"name">) -> impl Responder {
    format!("Hello {}!", *name)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    HttpServer::new(
        App::new()
            .service(web::get("/").to(index))
            .service(web::get("/:name").to(greet))
    )
    .bind(("127.0.0.1", 8080))
    .run()?;

    Ok(())
}
```

## FAQ

Q: Why the name 'enrgy'?

A: I was trying to come up with a name and miss-spelled energy.

Q: Why does this exist?

A: I needed a lightweight `actix-web` like server library, I also wanted to mess around with unstable features.
