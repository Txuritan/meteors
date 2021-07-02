use {
    crate::{
        extensions::Extensions, route::Route, service::BoxedService, Data, Error, HttpRequest,
        HttpResponse, Method, Middleware,
    },
    path_tree::PathTree,
    std::{collections::BTreeMap, sync::Arc},
};

type InnerRoute = BoxedService<HttpRequest, HttpResponse, Error>;

pub(crate) fn not_found() -> Result<HttpResponse, Error> {
    todo!()
}

#[derive(Clone)]
pub struct BuiltApp {
    pub(crate) tree: Arc<BTreeMap<Method, PathTree<Arc<InnerRoute>>>>,
    pub(crate) data: Arc<Extensions>,
    pub(crate) middleware: Arc<Vec<Box<dyn Middleware + Send + Sync + 'static>>>,
    pub(crate) not_found: Arc<InnerRoute>,
}

pub struct App {
    tree: BTreeMap<Method, PathTree<Arc<InnerRoute>>>,
    data: Extensions,
    middleware: Vec<Box<dyn Middleware + Send + Sync + 'static>>,
    not_found: Arc<InnerRoute>,
}

impl App {
    pub fn data<T>(mut self, data: Arc<T>) -> Self
    where
        T: Send + Sync + 'static,
    {
        self.data.insert(Data { data });

        self
    }

    pub fn wrap<M>(mut self, middleware: M) -> Self
    where
        M: Middleware + Send + Sync + 'static,
    {
        self.middleware.push(Box::new(middleware));

        self
    }

    pub fn service(mut self, route: Route<'_>) -> Self {
        let node = self.tree.entry(route.method).or_insert_with(PathTree::new);

        node.insert(route.path, Arc::new(route.service));

        self
    }

    pub fn build(self) -> BuiltApp {
        BuiltApp {
            tree: Arc::new(self.tree),
            data: Arc::new(self.data),
            middleware: Arc::new(self.middleware),
            not_found: self.not_found,
        }
    }
}
