use {
    crate::{
        handler::HandlerService, route::Route, service::BoxedService, service::Service as _, Data,
        Error, HttpRequest, HttpResponse, Method, Middleware,
    },
    path_tree::PathTree,
    std::{
        any::{Any, TypeId},
        collections::BTreeMap,
        sync::Arc,
    },
};

type InnerRoute = BoxedService<HttpRequest, HttpResponse, Error>;

pub(crate) fn not_found() -> Result<HttpResponse, Error> {
    todo!()
}

#[derive(Clone)]
pub struct Router {
    tree: Arc<BTreeMap<Method, PathTree<Arc<InnerRoute>>>>,
    data: Arc<Extensions>,
    middleware: Arc<Vec<Box<dyn Middleware + Send + Sync + 'static>>>,
    not_found: Arc<InnerRoute>,
}

impl Router {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> RouterBuilder {
        RouterBuilder {
            tree: BTreeMap::new(),
            data: Extensions::new(),
            middleware: Vec::new(),
            not_found: Arc::new(BoxedService::new(HandlerService::new(not_found))),
        }
    }

    pub fn handle(&self, request: tiny_http::Request) -> Result<(), Error> {
        let url = request.url().to_string();

        let (url, raw_query) = url.split_at(url.find('?').unwrap_or_else(|| url.len()));

        let method = Method::from(request.method());

        let mut req = HttpRequest {
            inner: request,
            data: Arc::clone(&self.data),
            ext: Extensions::new(),
            parameters: BTreeMap::new(),
            query: BTreeMap::new(),
            raw_query: raw_query.to_string(),
        };

        for middleware in &*self.middleware {
            middleware.before(&mut req);
        }

        let query = form_urlencoded::parse(raw_query.trim_start_matches('?').as_bytes())
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect::<BTreeMap<_, _>>();

        let (service, parameters) = self
            .tree
            .get(&method)
            .and_then(|tree| tree.find(url))
            .map(|(service, parameters)| {
                (
                    Arc::clone(service),
                    parameters
                        .into_iter()
                        .map(|(key, value)| (key.to_string(), value.to_string()))
                        .collect::<BTreeMap<_, _>>(),
                )
            })
            .unwrap_or_else(|| (self.not_found.clone(), BTreeMap::new()));

        req.parameters = parameters;
        req.query = query;

        let (response, request) = service.call(&mut req).map(|res| (res, req))?;

        for middleware in &*self.middleware {
            middleware.after(&request, &response);
        }

        request.inner.respond(response.inner)?;

        Ok(())
    }
}

pub struct RouterBuilder {
    tree: BTreeMap<Method, PathTree<Arc<InnerRoute>>>,
    data: Extensions,
    middleware: Vec<Box<dyn Middleware + Send + Sync + 'static>>,
    not_found: Arc<InnerRoute>,
}

impl RouterBuilder {
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

    pub fn build(self) -> Router {
        Router {
            tree: Arc::new(self.tree),
            data: Arc::new(self.data),
            middleware: Arc::new(self.middleware),
            not_found: self.not_found,
        }
    }
}

pub struct Extensions {
    inner: BTreeMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl Extensions {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::default(),
        }
    }

    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        fn downcast_owned<T: 'static>(boxed: Box<dyn Any + Send + Sync>) -> Option<T> {
            boxed.downcast().ok().map(|boxed| *boxed)
        }

        self.inner
            .insert(TypeId::of::<T>(), Box::new(val))
            .and_then(downcast_owned)
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        fn downcast_ref<T: 'static>(boxed: &Box<dyn Any + Send + Sync>) -> Option<&T> {
            boxed.downcast_ref()
        }

        self.inner.get(&TypeId::of::<T>()).and_then(downcast_ref)
    }
}

impl Default for Extensions {
    fn default() -> Self {
        Self::new()
    }
}
