use std::{
    fmt,
    io::{self, BufReader},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use crate::{
    app::ArcRouter,
    extractor::param::ParsedParams,
    http::{self, headers::ACCEPT_ENCODING, HttpRequest},
    middleware::Middleware as _,
    service::Service,
    utils::{signal, thread_pool::ThreadPool, ArrayMap},
    Router, Error,
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

pub struct Unbound;

pub struct Server<Addr> {
    close: Arc<AtomicBool>,

    workers: Vec<JoinHandle<()>>,

    addr: Addr,

    app: ArcRouter,
}

impl Server<Unbound> {
    pub fn new(app: Router) -> Self {
        Self {
            close: Arc::new(AtomicBool::new(false)),
            workers: Vec::with_capacity(4),
            addr: Unbound,
            app: app.build(),
        }
    }

    pub fn bind<A>(self, addr: A) -> Server<SocketAddr>
    where
        A: Into<SocketAddr>,
    {
        Server {
            close: self.close,
            workers: self.workers,
            addr: addr.into(),
            app: self.app,
        }
    }
}

impl Server<SocketAddr> {
    pub fn run(self) -> Result<(), RunError> {
        signal::set_handler({
            let close = Arc::clone(&self.close);

            move || {
                close.store(true, Ordering::SeqCst);
            }
        })?;

        let listener = TcpListener::bind(self.addr)?;

        let (pool, sender) = ThreadPool::new(4, Arc::clone(&self.close), Self::thread_pool_handler);

        thread::spawn({
            let app = self.app.clone();

            move || {
                while let Ok((stream, addr)) = listener.accept() {
                    if sender.send((app.clone(), stream, addr)).is_err() {
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
    Enrgy(Error),
    Http(http::HttpError),
    Io(io::Error),
    ParseInt(std::num::ParseIntError),
    Utf8(std::string::FromUtf8Error),
}

impl const From<Error> for ThreadError {
    fn from(v: Error) -> Self {
        Self::Enrgy(v)
    }
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
    fn thread_pool_handler((app, mut stream, _addr): (ArcRouter, TcpStream, SocketAddr)) {
        fn run(app: ArcRouter, stream: &mut TcpStream) {
            if let Err(err) = Server::thread_handle(app, stream) {
                log::error!("unable to handle thread");

                match err {
                    ThreadError::Enrgy(err) => log::error!("route handler error: {:?}", err),
                    ThreadError::Http(err) => log::error!("invalid http: {:?}", err),
                    ThreadError::Io(err) => log::error!("{}", err),
                    ThreadError::ParseInt(err) => log::error!("{}", err),
                    ThreadError::Utf8(err) => log::error!("{}", err),
                }
            }
        }

        run(app, &mut stream);
    }

    fn thread_handle(app: ArcRouter, stream: &mut TcpStream) -> Result<(), ThreadError> {
        let mut buf_reader = BufReader::new(stream);
        let mut request = HttpRequest::from_buf_reader(Arc::clone(&app.data), &mut buf_reader)?;
        let stream = buf_reader.into_inner();

        // let (header_data, body) = http::read_request(stream)?;

        let (service, params) = app
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
            .unwrap_or_else(|| (app.default_service.clone(), ParsedParams(ArrayMap::new())));

        request.extensions.insert(params);

        let compress = if let Some(header) = request.headers.get(&ACCEPT_ENCODING) {
            header.contains("deflate")
        } else {
            false
        };

        for middleware in &*app.middleware {
            middleware.before(&mut request);
        }

        let mut response = service.call(&mut request)?;

        for middleware in &*app.middleware {
            response = middleware.after(&request, response);
        }

        http::write_response(response, compress, stream)?;

        Ok(())
    }
}
