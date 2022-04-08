// this is all taken from axum-core, just converted over to use enrgy's http types

mod into_response;
mod into_response_parts;

use crate::http::{headers, HttpResponse};

pub use self::{
    into_response::IntoResponse,
    into_response_parts::{IntoResponseParts, ResponseParts},
};

pub struct Html<T>(pub T);

impl<T> IntoResponse for Html<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> HttpResponse {
        (
            [(
                headers::CONTENT_TYPE,
                headers::HttpHeaderValue::new("text/html; charset=utf-8".to_string()),
            )],
            self.0,
        )
            .into_response()
    }
}
