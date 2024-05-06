use std::marker::PhantomData;

use crate::{
    extractor::Extractor,
    http::{HttpRequest, HttpResponse},
    response::IntoResponse,
    service::Service,
};

pub trait Handler<T> {
    fn call(&self, req: &mut HttpRequest) -> HttpResponse;

    fn empty(&self, _params: T) {}
}

pub struct HandlerService<F, T>
where
    F: Handler<T>,
{
    inner: F,
    _phantom: PhantomData<T>,
}

impl<F, T> HandlerService<F, T>
where
    F: Handler<T>,
{
    pub(crate) const fn new(fun: F) -> Self {
        Self {
            inner: fun,
            _phantom: PhantomData,
        }
    }
}

impl<F, T> Service<HttpRequest> for HandlerService<F, T>
where
    F: Handler<T>,
{
    type Response = HttpResponse;

    type Error = HttpResponse;

    fn call(&self, req: &mut HttpRequest) -> Result<Self::Response, Self::Error> {
        let res = self.inner.call(req);

        Ok(res.into_response())
    }
}

impl<FUN, R> Handler<()> for FUN
where
    FUN: Fn() -> R,
    R: IntoResponse,
{
    #[allow(non_snake_case)]
    fn call(&self, _req: &mut HttpRequest) -> HttpResponse {
        (self)().into_response()
    }
}

macro_rules! tuple (
    { $($param:ident)* } => {
        impl<FUN, R, $($param,)*> Handler<($($param,)*)> for FUN
        where
            FUN: Fn($($param),*) -> R,
            R: IntoResponse,
            $($param: Extractor,)*
        {
            #[allow(non_snake_case)]
            fn call(&self, req: &mut HttpRequest) -> HttpResponse {
                $(
                    let $param = match $param::extract(req) {
                        Ok(v) => v,
                        Err(err) => return err.into_response(),
                    };
                )*
                (self)($($param),*).into_response()
            }
        }
    }
);

tuple! { A }
tuple! { A B }
tuple! { A B C }
tuple! { A B C D }
tuple! { A B C D E }
tuple! { A B C D E F }
tuple! { A B C D E F G }
tuple! { A B C D E F G H }
tuple! { A B C D E F G H I }
tuple! { A B C D E F G H I J }
tuple! { A B C D E F G H I J K }
tuple! { A B C D E F G H I J K L }
