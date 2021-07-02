pub trait Service<Request> {
    type Response;

    type Error;

    fn call(&self, req: &mut Request) -> Result<Self::Response, Self::Error>;
}

pub struct BoxedService<Request, Response, Error> {
    inner: Box<dyn Service<Request, Response = Response, Error = Error> + Send + Sync>,
}

impl<Request, Response, Error> BoxedService<Request, Response, Error> {
    pub(crate) fn new<T>(inner: T) -> Self
    where
        T: Service<Request, Response = Response, Error = Error> + Send + Sync + 'static,
    {
        Self {
            inner: Box::new(inner),
        }
    }
}

impl<Request, Response, Error> Service<Request> for BoxedService<Request, Response, Error> {
    type Response = Response;

    type Error = Error;

    fn call(&self, req: &mut Request) -> Result<Self::Response, Self::Error> {
        (self.inner).call(req)
    }
}
