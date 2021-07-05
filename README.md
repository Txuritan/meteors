# enrgy

An (nightly) insecure lightweight synchronous Actix-like HTTP server.

*WARNING*: Do not allow access to this server from the open internet, it little to no security measures.

## Example

```rust
use enrgy::{get, App, HttpServer, Param, Responder};

fn index() -> impl Responder {
    "Hello World!"
}

fn greet(name: Param<"name">) -> impl Responder {
    format!("Hello {}!", &name)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    HttpServer::new(
        App::new()
            .service(get("/").to(index))
            .service(get("/:name").to(greet))
    )
    .bind(("127.0.0.1", 8080))
    .run()?;

    Ok(())
}
```

## FAQ

Q: Why the name 'enrgy'?

A: I was trying to come up with a name and miss-spelled energy.
