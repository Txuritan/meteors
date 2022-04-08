use std::convert::TryInto;

use crate::{
    extensions::Extensions,
    http::{self, headers::HttpHeaderMap, HttpResponse, StatusCode},
    response::IntoResponse,
};

pub struct ResponseParts {
    pub(crate) res: HttpResponse,
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

impl<K, V, const N: usize> IntoResponseParts for [(K, V); N]
where
    K: TryInto<http::HttpHeaderName>,
    K::Error: std::fmt::Display,
    V: TryInto<http::HttpHeaderValue>,
    V::Error: std::fmt::Display,
{
    type Error = TryIntoHeaderError<K::Error, V::Error>;

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        for (key, value) in self {
            let key = key.try_into().map_err(TryIntoHeaderError::key)?;
            let value = value.try_into().map_err(TryIntoHeaderError::value)?;
            res.headers_mut().insert(key, value);
        }

        Ok(res)
    }
}

#[derive(Debug)]
pub struct TryIntoHeaderError<K, V> {
    kind: TryIntoHeaderErrorKind<K, V>,
}

impl<K, V> TryIntoHeaderError<K, V> {
    fn key(err: K) -> Self {
        Self {
            kind: TryIntoHeaderErrorKind::Key(err),
        }
    }

    fn value(err: V) -> Self {
        Self {
            kind: TryIntoHeaderErrorKind::Value(err),
        }
    }
}

#[derive(Debug)]
enum TryIntoHeaderErrorKind<K, V> {
    Key(K),
    Value(V),
}

impl<K, V> IntoResponse for TryIntoHeaderError<K, V>
where
    K: std::fmt::Display,
    V: std::fmt::Display,
{
    fn into_response(self) -> HttpResponse {
        match self.kind {
            TryIntoHeaderErrorKind::Key(inner) => {
                (StatusCode::INTERNAL_SERVER_ERROR, inner.to_string()).into_response()
            }
            TryIntoHeaderErrorKind::Value(inner) => {
                (StatusCode::INTERNAL_SERVER_ERROR, inner.to_string()).into_response()
            }
        }
    }
}

impl<K, V> std::fmt::Display for TryIntoHeaderError<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            TryIntoHeaderErrorKind::Key(_) => write!(f, "failed to convert key to a header name"),
            TryIntoHeaderErrorKind::Value(_) => {
                write!(f, "failed to convert value to a header value")
            }
        }
    }
}

impl<K, V> std::error::Error for TryIntoHeaderError<K, V>
where
    K: std::error::Error + 'static,
    V: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            TryIntoHeaderErrorKind::Key(inner) => Some(inner),
            TryIntoHeaderErrorKind::Value(inner) => Some(inner),
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
