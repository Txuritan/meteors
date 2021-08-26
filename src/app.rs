use {
    crate::{
        extensions::Extensions,
        handler::HandlerService,
        http::Method,
        middleware::Middleware,
        route::{self, Route},
        service::BoxedService,
        utils::{ArrayMap, PathTree},
        web, Error, HttpRequest, HttpResponse,
    },
    std::sync::Arc,
};

type InnerRoute = BoxedService<HttpRequest, HttpResponse, Error>;

#[derive(Clone)]
pub struct BuiltApp {
    pub(crate) tree: Arc<ArrayMap<Method, PathTree<Arc<InnerRoute>>, 9>>,
    pub(crate) data: Arc<Extensions>,
    pub(crate) middleware: Arc<Vec<Box<dyn Middleware + Send + Sync + 'static>>>,
    pub(crate) default_service: Arc<InnerRoute>,
}

pub struct App {
    tree: ArrayMap<Method, PathTree<Arc<InnerRoute>>, 9>,
    data: Extensions,
    middleware: Vec<Box<dyn Middleware + Send + Sync + 'static>>,
    default_service: Arc<InnerRoute>,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn data<T>(mut self, data: Arc<T>) -> Self
    where
        T: Send + Sync + 'static,
    {
        self.data.insert(web::Data { data });

        self
    }

    pub fn wrap<M>(mut self, middleware: M) -> Self
    where
        M: Middleware + Send + Sync + 'static,
    {
        self.middleware.push(box middleware);

        self
    }

    pub fn service(mut self, route: Route<'_>) -> Self {
        let node = if let Some(node) = self.tree.get_mut(route.method) {
            node
        } else {
            self.tree.insert(route.method, PathTree::new());

            unsafe { self.tree.get_mut(route.method).unwrap_unchecked() }
        };

        node.insert(route.path, Arc::new(route.service));

        self
    }

    pub fn default_service(mut self, service: Route<'static>) -> Self {
        self.default_service = Arc::new(service.service);

        self
    }

    pub fn build(self) -> BuiltApp {
        BuiltApp {
            tree: Arc::new(self.tree),
            data: Arc::new(self.data),
            middleware: Arc::new(self.middleware),
            default_service: self.default_service,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
            tree: ArrayMap::new(),
            data: Extensions::new(),
            middleware: Vec::new(),
            default_service: Arc::new(BoxedService::new(HandlerService::new(route::not_found))),
        }
    }
}
