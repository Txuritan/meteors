use std::marker::PhantomData;

use crate::{
    extractor::Extractor,
    http::{HttpRequest, HttpResponse},
    response::IntoResponse,
    service::Service,
    Error,
};

pub trait Handler<T, R>
where
    T: Extractor<Error = Error>,
    R: IntoResponse,
{
    fn call(&self, param: T) -> R;
}

pub struct HandlerService<F, T, R>
where
    F: Handler<T, R>,
    T: Extractor<Error = Error>,
    R: IntoResponse,
{
    inner: F,
    _phantom: PhantomData<(T, R)>,
}

impl<F, T, R> HandlerService<F, T, R>
where
    F: Handler<T, R>,
    T: Extractor<Error = Error>,
    R: IntoResponse,
{
    pub(crate) const fn new(fun: F) -> Self {
        Self {
            inner: fun,
            _phantom: PhantomData,
        }
    }
}

impl<F, T, R> Service<HttpRequest> for HandlerService<F, T, R>
where
    F: Handler<T, R>,
    T: Extractor<Error = Error>,
    R: IntoResponse,
{
    type Response = HttpResponse;

    type Error = Error;

    fn call(&self, req: &mut HttpRequest) -> Result<Self::Response, Self::Error> {
        let param = T::extract(req)?;

        let res = self.inner.call(param);

        Ok(res.into_response())
    }
}

impl<FUN, R> Handler<(), R> for FUN
where
    FUN: Fn() -> R,
    R: IntoResponse,
{
    #[allow(non_snake_case)]
    fn call(&self, _param: ()) -> R {
        (self)()
    }
}

macro_rules! tuple (
    { $($param:ident)* } => {
        impl<FUN, $($param,)* R> Handler<($($param,)*), R> for FUN
        where
            FUN: Fn($($param),*) -> R,
            $($param: Extractor<Error = Error>,)*
            R: IntoResponse,
        {
            #[allow(non_snake_case)]
            fn call(&self, ($($param,)*): ($($param,)*)) -> R {
                (self)($($param,)*)
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
