use std::{
    fmt,
    io::{self, BufReader},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use crate::{
    dev::BoxedService,
    extensions::Extensions,
    extractor::{self, param::ParsedParams},
    handler::HandlerService,
    http::{self, headers::ACCEPT_ENCODING, HttpMethod, HttpRequest, HttpResponse},
    middleware::{BoxedMiddleware, Middleware as _},
    route::{self, Route},
    service::Service,
    utils::{signal, thread_pool::ThreadPool, ArrayMap, PathTree},
};

#[derive(Debug)]
pub enum RunError {
    Io(std::io::Error),
    Signal(signal::Error),
}

impl const From<std::io::Error> for RunError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl const From<signal::Error> for RunError {
    fn from(err: signal::Error) -> Self {
        Self::Signal(err)
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunError::Io(err) => err.fmt(f),
            RunError::Signal(err) => err.fmt(f),
        }
    }
}

impl std::error::Error for RunError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RunError::Io(err) => Some(err),
            RunError::Signal(err) => Some(err),
        }
    }
}

type InnerRoute = BoxedService<HttpRequest, HttpResponse, HttpResponse>;
type RouterTree = ArrayMap<HttpMethod, PathTree<Arc<InnerRoute>>, 9>;
type Middleware = Vec<BoxedMiddleware<HttpRequest, HttpResponse>>;

#[derive(Clone)]
struct Shared {
    data: Arc<Extensions>,
    default: Arc<InnerRoute>,
    tree: Arc<RouterTree>,
    middleware: Arc<Middleware>,
}

pub struct Unbound;

pub struct Server<Addr> {
    close: Arc<AtomicBool>,

    addr: Addr,

    data: Extensions,
    default: Arc<InnerRoute>,
    tree: RouterTree,
    middleware: Middleware,
}

impl<Addr> Server<Addr> {
    pub fn data<T>(mut self, data: Arc<T>) -> Self
    where
        T: Send + Sync + 'static,
    {
        self.data.insert(extractor::Data { data });

        self
    }

    pub fn wrap<M>(mut self, middleware: M) -> Self
    where
        M: crate::middleware::Middleware<HttpRequest, HttpResponse> + Send + Sync + 'static,
    {
        self.middleware.push(BoxedMiddleware::new(middleware));

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

    pub fn default(mut self, service: Route<'static>) -> Self {
        self.default = Arc::new(service.service);

        self
    }
}

impl Server<Unbound> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            close: Arc::new(AtomicBool::new(false)),
            addr: Unbound,
            data: Extensions::new(),
            default: Arc::new(BoxedService::new(HandlerService::new(route::not_found))),
            tree: RouterTree::new(),
            middleware: Middleware::new(),
        }
    }

    pub fn bind<A>(self, addr: A) -> Server<SocketAddr>
    where
        A: Into<SocketAddr>,
    {
        Server {
            close: self.close,
            addr: addr.into(),
            data: self.data,
            default: self.default,
            tree: self.tree,
            middleware: self.middleware,
        }
    }
}

impl Server<SocketAddr> {
    pub fn run(self) -> Result<(), RunError> {
        let Server {
            close,
            addr,
            data,
            default,
            tree,
            middleware,
        } = self;

        let shared = Shared {
            data: Arc::new(data),
            default,
            tree: Arc::new(tree),
            middleware: Arc::new(middleware),
        };

        signal::set_handler({
            let close = Arc::clone(&close);

            move || {
                close.store(true, Ordering::SeqCst);
            }
        })?;

        let listener = TcpListener::bind(addr)?;

        let (pool, sender) = ThreadPool::<_, 4>::new(Arc::clone(&close), Self::thread_pool_handler);

        thread::spawn({
            move || {
                while let Ok((stream, addr)) = listener.accept() {
                    if sender.send((shared.clone(), stream, addr)).is_err() {
                        break;
                    }
                }
            }
        });

        pool.join();

        Ok(())
    }
}

enum ThreadError {
    Http(http::HttpError),
    Io(io::Error),
    ParseInt(std::num::ParseIntError),
    Utf8(std::string::FromUtf8Error),
}

impl const From<http::HttpError> for ThreadError {
    fn from(v: http::HttpError) -> Self {
        Self::Http(v)
    }
}

impl const From<io::Error> for ThreadError {
    fn from(v: io::Error) -> Self {
        Self::Io(v)
    }
}

impl const From<std::num::ParseIntError> for ThreadError {
    fn from(v: std::num::ParseIntError) -> Self {
        Self::ParseInt(v)
    }
}

impl const From<std::string::FromUtf8Error> for ThreadError {
    fn from(v: std::string::FromUtf8Error) -> Self {
        Self::Utf8(v)
    }
}

impl Server<SocketAddr> {
    fn thread_pool_handler((shared, mut stream, _addr): (Shared, TcpStream, SocketAddr)) {
        fn run(shared: Shared, stream: &mut TcpStream) {
            if let Err(err) = Server::thread_handle(shared, stream) {
                log::error!("unable to handle thread");

                match err {
                    ThreadError::Http(err) => log::error!("invalid http: {:?}", err),
                    ThreadError::Io(err) => log::error!("{}", err),
                    ThreadError::ParseInt(err) => log::error!("{}", err),
                    ThreadError::Utf8(err) => log::error!("{}", err),
                }
            }
        }

        run(shared, &mut stream);
    }

    fn thread_handle(shared: Shared, stream: &mut TcpStream) -> Result<(), ThreadError> {
        let mut buf_reader = BufReader::new(stream);
        let mut request = HttpRequest::from_buf_reader(Arc::clone(&shared.data), &mut buf_reader)?;
        let stream = buf_reader.into_inner();

        let (service, params) = shared
            .tree
            .get(&request.method)
            .and_then(|tree| tree.find(&request.uri.path))
            .map(|(service, params)| {
                let mut map = ParsedParams(ArrayMap::new());

                for (key, value) in params.into_iter() {
                    map.0.insert(key.to_string(), value.to_string());
                }

                (Arc::clone(service), map)
            })
            .unwrap_or_else(|| (shared.default.clone(), ParsedParams(ArrayMap::new())));

        request.extensions.insert(params);

        let compress = if let Some(header) = request.headers.get(&ACCEPT_ENCODING) {
            header.contains("deflate")
        } else {
            false
        };

        for middleware in &*shared.middleware {
            middleware.before(&mut request);
        }

        let mut response = match service.call(&mut request) {
            Ok(res) => res,
            Err(err) => err,
        };

        for middleware in &*shared.middleware {
            response = middleware.after(&request, response);
        }

        http::write_response(response, compress, stream)?;

        Ok(())
    }
}
