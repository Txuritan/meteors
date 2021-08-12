pub macro res {
    (200; $body:expr) => {
        ::enrgy::HttpResponse::ok()
            .header(::enrgy::http::headers::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(::opal::Template::render_into_string($body)?)
    },
    (404) => {
        ::enrgy::HttpResponse::not_found()
            .header(::enrgy::http::headers::CONTENT_TYPE, "text/html; charset=utf-8")
            .body({
                let rendered = ::opal::Template::render_into_string(crate::templates::Layout::not_found());

                debug_assert!(rendered.is_ok());

                unsafe { rendered.unwrap_unchecked() }
            })
    },
    (503) => {
        ::enrgy::HttpResponse::internal_server_error()
            .header(::enrgy::http::headers::CONTENT_TYPE, "text/html; charset=utf-8")
            .body({
                let rendered = ::opal::Template::render_into_string(crate::templates::Layout::internal_server_error());

                debug_assert!(rendered.is_ok());

                unsafe { rendered.unwrap_unchecked() }
            })
    },
}
