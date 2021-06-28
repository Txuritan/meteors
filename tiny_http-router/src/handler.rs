use {
    crate::{
        extractor::Extractor,
        http::{HttpRequest, HttpResponse},
        service::Service,
        Error,
    },
    std::marker::PhantomData,
};

pub trait Handler<T>
where
    T: Extractor,
{
    fn call(&self, param: T) -> Result<HttpResponse, Error>;
}

pub struct HandlerService<F, T>
where
    F: Handler<T>,
    T: Extractor,
{
    inner: F,
    _phantom: PhantomData<T>,
}

impl<F, T> HandlerService<F, T>
where
    F: Handler<T>,
    T: Extractor,
{
    pub(crate) fn new(fun: F) -> Self {
        Self {
            inner: fun,
            _phantom: PhantomData,
        }
    }
}

impl<F, T> Service<HttpRequest> for HandlerService<F, T>
where
    F: Handler<T>,
    T: Extractor<Error = Error>,
{
    type Response = HttpResponse;

    type Error = Error;

    fn call(&self, req: &mut HttpRequest) -> Result<Self::Response, Self::Error> {
        let param = T::extract(req)?;

        self.inner.call(param)
    }
}

impl<FUN> Handler<()> for FUN
where
    FUN: Fn() -> Result<HttpResponse, Error>,
{
    #[allow(non_snake_case)]
    fn call(&self, _param: ()) -> Result<HttpResponse, Error> {
        (self)()
    }
}

macro_rules! tuple (
    { $($param:ident)* } => {
        impl<FUN, $($param,)*> Handler<($($param,)*)> for FUN
        where
            FUN: Fn($($param),*) -> Result<HttpResponse, Error>,
            $($param: Extractor<Error = Error>,)*
        {
            #[allow(non_snake_case)]
            fn call(&self, ($($param,)*): ($($param,)*)) -> Result<HttpResponse, Error> {
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
