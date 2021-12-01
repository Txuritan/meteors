pub trait Middleware<Req, Res> {
    fn before(&self, req: &mut Req);
    fn after(&self, req: &Req, res: Res) -> Res;
}

pub struct BoxedMiddleware<Req, Res> {
    inner: Box<dyn Middleware<Req, Res> + Send + Sync + 'static>,
}

impl<Req, Res> BoxedMiddleware<Req, Res> {
    pub fn new<T>(middleware: T) -> Self
    where
        T: Middleware<Req, Res> + Send + Sync + 'static,
    {
        Self {
            inner: box middleware,
        }
    }
}

impl<Req, Res> const Middleware<Req, Res> for BoxedMiddleware<Req, Res> {
    fn before(&self, req: &mut Req) {
        self.inner.before(req)
    }

    fn after(&self, req: &Req, res: Res) -> Res {
        self.inner.after(req, res)
    }
}
