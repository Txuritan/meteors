// this is all taken from axum-core, just converted over to use enrgy's http types

mod into_response;
mod into_response_parts;

use crate::http::{headers, HttpResponse};

pub use self::{
    into_response::IntoResponse,
    into_response_parts::{IntoResponseParts, ResponseParts},
};

macro_rules! impl_responder {
    ($( $name:ident => $mime:expr , )*) => {
        $(
            pub struct $name<T>(pub T);

            impl<T> IntoResponse for $name<T>
            where
                T: IntoResponse,
            {
                fn into_response(self) -> HttpResponse {
                    (
                        [(
                            headers::CONTENT_TYPE,
                            headers::HttpHeaderValue::new_static($mime),
                        )],
                        self.0,
                    )
                        .into_response()
                }
            }
        )*
    };
}

impl_responder! {
    Atom => "application/atom+xml; charset=utf-8",
    Html => "text/html; charset=utf-8",
    Xml => "application/xml; charset=utf-8",
}
