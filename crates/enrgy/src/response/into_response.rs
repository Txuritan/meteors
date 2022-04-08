use std::convert::{Infallible, TryInto};

use crate::{
    http::{self, HttpResponse, StatusCode},
    response::{IntoResponseParts, ResponseParts},
};

pub trait IntoResponse {
    fn into_response(self) -> HttpResponse;
}

impl IntoResponse for HttpResponse {
    fn into_response(self) -> HttpResponse {
        self
    }
}

impl IntoResponse for () {
    fn into_response(self) -> HttpResponse {
        HttpResponse::new(StatusCode::OK)
    }
}

impl IntoResponse for Infallible {
    fn into_response(self) -> HttpResponse {
        match self {}
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> HttpResponse {
        let mut res = ().into_response();
        *res.status_mut() = self;
        res
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> HttpResponse {
        HttpResponse::ok().body(self)
    }
}

impl IntoResponse for &'static [u8] {
    fn into_response(self) -> HttpResponse {
        HttpResponse::ok().body(self)
    }
}

impl IntoResponse for String {
    fn into_response(self) -> HttpResponse {
        HttpResponse::ok().body(self)
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> HttpResponse {
        HttpResponse::ok().body(self)
    }
}

impl<R> IntoResponse for (StatusCode, R)
where
    R: IntoResponse,
{
    fn into_response(self) -> HttpResponse {
        let mut res = self.1.into_response();
        *res.status_mut() = self.0;
        res
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self) -> HttpResponse {
        match self {
            Ok(value) => value.into_response(),
            Err(err) => err.into_response(),
        }
    }
}

impl<K, V, const N: usize> IntoResponse for [(K, V); N]
where
    K: TryInto<http::HttpHeaderName>,
    K::Error: std::fmt::Display,
    V: TryInto<http::HttpHeaderValue>,
    V::Error: std::fmt::Display,
{
    fn into_response(self) -> HttpResponse {
        let mut res = ().into_response();

        for (key, value) in self {
            let key = match key.try_into() {
                Ok(key) => key,
                Err(err) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
                }
            };

            let value = match value.try_into() {
                Ok(value) => value,
                Err(err) => {
                    return (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response()
                }
            };

            res.headers_mut().insert(key, value);
        }

        res
    }
}

macro_rules! impl_into_response {
    ( $($ty:ident),* $(,)? ) => {
        #[allow(non_snake_case)]
        impl<R, $($ty,)*> IntoResponse for ($($ty),*, R)
        where
            $( $ty: IntoResponseParts, )*
            R: IntoResponse,
        {
            fn into_response(self) -> HttpResponse {
                let ($($ty),*, res) = self;

                let res = res.into_response();
                let parts = ResponseParts { res };

                $(
                    let parts = match $ty.into_response_parts(parts) {
                        Ok(parts) => parts,
                        Err(err) => {
                            return err.into_response();
                        }
                    };
                )*

                parts.res
            }
        }

        #[allow(non_snake_case)]
        impl<R, $($ty,)*> IntoResponse for (StatusCode, $($ty),*, R)
        where
            $( $ty: IntoResponseParts, )*
            R: IntoResponse,
        {
            fn into_response(self) -> HttpResponse {
                let (status, $($ty),*, res) = self;

                let res = res.into_response();
                let parts = ResponseParts { res };

                $(
                    let parts = match $ty.into_response_parts(parts) {
                        Ok(parts) => parts,
                        Err(err) => {
                            return err.into_response();
                        }
                    };
                )*

                let mut res = parts.res;
                *res.status_mut() = status;
                res
            }
        }
    }
}

all_the_tuples!(impl_into_response);
