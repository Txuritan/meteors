use {
    crate::{
        extractor::{Extractor, ExtractorError},
        HttpRequest, HttpResponse,
        service::Service,
        Responder
    },
    std::marker::PhantomData,
};

pub enum HandlerError {
    Extractor(ExtractorError),
}

impl From<ExtractorError> for HandlerError {
    fn from(err: ExtractorError) -> Self {
        HandlerError::Extractor(err)
    }
}

pub trait Handler<T, R>
where
    T: Extractor,
    R: Responder,
{
    fn call(&self, param: T) -> R;
}

pub struct HandlerService<F, T, R>
where
    F: Handler<T, R>,
    T: Extractor,
    R: Responder,
{
    inner: F,
    _phantom: PhantomData<(T, R)>,
}

impl<F, T, R> HandlerService<F, T, R>
where
    F: Handler<T, R>,
    T: Extractor,
    R: Responder,
{
    pub(crate) fn new(fun: F) -> Self {
        Self {
            inner: fun,
            _phantom: PhantomData,
        }
    }
}

impl<F, T, R> Service<HttpRequest> for HandlerService<F, T, R>
where
    F: Handler<T, R>,
    T: Extractor<Error = ExtractorError>,
    R: Responder,
{
    type Response = HttpResponse;

    type Error = HandlerError;

    fn call(&self, req: &mut HttpRequest) -> Result<Self::Response, Self::Error> {
        let param = T::extract(req)?;

        let res = self.inner.call(param);

        Ok(res.respond_to(&*req))
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
            $($param: Extractor<Error = ExtractorError>,)*
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
