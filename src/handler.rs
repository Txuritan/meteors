use std::marker::PhantomData;

use crate::{
    extractor::Extractor,
    http::{HttpRequest, HttpResponse},
    service::Service,
    Error, Responder,
};

pub trait Handler<T, R>
where
    T: Extractor<Error = Error>,
    R: Responder,
{
    fn call(&self, param: T) -> R;
}

pub struct HandlerService<F, T, R>
where
    F: Handler<T, R>,
    T: Extractor<Error = Error>,
    R: Responder,
{
    inner: F,
    _phantom: PhantomData<(T, R)>,
}

impl<F, T, R> HandlerService<F, T, R>
where
    F: Handler<T, R>,
    T: Extractor<Error = Error>,
    R: Responder,
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
    R: Responder,
{
    type Response = HttpResponse;

    type Error = Error;

    fn call(&self, req: &mut HttpRequest) -> Result<Self::Response, Self::Error> {
        let param = T::extract(req)?;

        let res = self.inner.call(param);

        res.respond_to(&*req)
    }
}

impl<FUN, R> Handler<(), R> for FUN
where
    FUN: Fn() -> R,
    R: Responder,
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
            R: Responder,
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
