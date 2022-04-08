//! A fork/modification of AXum's `IntoResponse` and `IntoResponseParts` system.

use crate::{
    extensions::Extensions,
    http::{headers::HttpHeaderMap, HttpResponse, StatusCode},
};

pub trait IntoResponse {
    fn into_response(self) -> HttpResponse;
}

impl IntoResponse for HttpResponse {
    fn into_response(self) -> HttpResponse {
        self
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

pub struct ResponseParts {
    res: HttpResponse,
}

impl ResponseParts {
    #[inline]
    pub const fn status(&self) -> StatusCode {
        self.res.status
    }

    #[inline]
    pub const fn status_mut(&mut self) -> &mut StatusCode {
        &mut self.res.status
    }

    #[inline]
    pub const fn headers(&self) -> &HttpHeaderMap {
        &self.res.headers
    }

    #[inline]
    pub const fn headers_mut(&mut self) -> &mut HttpHeaderMap {
        &mut self.res.headers
    }

    #[inline]
    pub const fn extensions(&self) -> &Extensions {
        &self.res.extensions
    }

    #[inline]
    pub const fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.res.extensions
    }
}

pub trait IntoResponseParts {
    type Error: IntoResponse;

    fn into_response_parts(self, res: ResponseParts) -> Result<ResponseParts, Self::Error>;
}

impl<T> IntoResponseParts for Option<T>
where
    T: IntoResponseParts,
{
    type Error = T::Error;

    fn into_response_parts(self, res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        match self {
            Some(t) => t.into_response_parts(res),
            None => Ok(res),
        }
    }
}

macro_rules! impl_into_response_parts {
    ( $( $ty:ident ),* $(,)? ) => {
        #[allow(non_snake_case)]
        impl<$($ty,)*> IntoResponseParts for ($($ty,)*)
        where
            $( $ty: IntoResponseParts, )*
        {
            type Error = HttpResponse;

            fn into_response_parts(self, res: ResponseParts) -> Result<ResponseParts, Self::Error> {
                let ($($ty,)*) = self;

                $(
                    let res = match $ty.into_response_parts(res) {
                        Ok(res) => res,
                        Err(err) => {
                            return Err(err.into_response());
                        }
                    };
                )*

                Ok(res)
            }
        }
    };
}

all_the_tuples!(impl_into_response_parts);
