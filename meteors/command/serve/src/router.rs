pub static PAGE_404: &str = r#"<!DOCTYPE html><html><head><title>404 | local archive</title><style>*{transition:all .6s}html{height:100%}body{font-family:sans-serif;color:#888;margin:0}#main{display:table;width:100%;height:100vh;text-align:center}.fof{display:table-cell;vertical-align:middle}.fof h1{font-size:50px;display:inline-block;padding-right:12px;animation:type .5s alternate infinite}@keyframes type{from{box-shadow:inset -3px 0 0 #888}to{box-shadow:inset -3px 0 0 transparent}}</style></head><body><div id="main"><div class="fof"><h1>Error 404</h1></div></div></body></html>"#;
pub static PAGE_503: &str = r#"<!DOCTYPE html><html><head><title>503 | local archive</title><style>*{transition:all .6s}html{height:100%}body{font-family:sans-serif;color:#888;margin:0}#main{display:table;width:100%;height:100vh;text-align:center}.fof{display:table-cell;vertical-align:middle}.fof h1{font-size:50px;display:inline-block;padding-right:12px;animation:type .5s alternate infinite}@keyframes type{from{box-shadow:inset -3px 0 0 #888}to{box-shadow:inset -3px 0 0 transparent}}</style></head><body><div id="main"><div class="fof"><h1>Error 503</h1></div></div></body></html>"#;

pub macro res {
    (200; $body:expr) => {
        ::enrgy::HttpResponse::ok()
            .header("Content-Type", "text/html; charset=utf-8")
            .body(::opal::Template::render_into_string($body)?)
    },
    (404) => {
        ::enrgy::HttpResponse::not_found()
            .header("Content-Type", "text/html; charset=utf-8")
            .body($crate::router::PAGE_404)
    },
    (503) => {
        ::enrgy::HttpResponse::internal_server_error()
            .header("Content-Type", "text/html; charset=utf-8")
            .body($crate::router::PAGE_503)
    },
}
