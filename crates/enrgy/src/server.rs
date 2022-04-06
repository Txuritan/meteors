use std::{
    fmt, io,
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use crate::{
    app::BuiltApp,
    extensions::Extensions,
    http::{self, headers::ACCEPT_ENCODING, HttpRequest},
    middleware::Middleware as _,
    service::Service,
    utils::{signal, thread_pool::ThreadPool, ArrayMap},
    App, Error,
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

pub struct HttpServer<Addr> {
    close: Arc<AtomicBool>,

    workers: Vec<JoinHandle<()>>,

    addr: Addr,

    app: Arc<BuiltApp>,
}

impl HttpServer<Unbound> {
    pub fn new(app: App) -> Self {
        Self {
            close: Arc::new(AtomicBool::new(false)),
            workers: Vec::with_capacity(4),
            addr: Unbound,
            app: Arc::new(app.build()),
        }
    }

    pub fn bind<A>(self, addr: A) -> HttpServer<SocketAddr>
    where
        A: Into<SocketAddr>,
    {
        HttpServer {
            close: self.close,
            workers: self.workers,
            addr: addr.into(),
            app: self.app,
        }
    }
}

impl HttpServer<SocketAddr> {
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
            let app = Arc::clone(&self.app);

            move || {
                while let Ok((stream, addr)) = listener.accept() {
                    if sender.send((Arc::clone(&app), stream, addr)).is_err() {
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

impl HttpServer<SocketAddr> {
    fn thread_pool_handler((app, mut stream, _addr): (Arc<BuiltApp>, TcpStream, SocketAddr)) {
        fn run(app: Arc<BuiltApp>, stream: &mut TcpStream) {
            if let Err(err) = HttpServer::thread_handle(app, stream) {
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

        if let Err(err) = stream.set_read_timeout(None) {
            log::error!(
                "internal tcp stream error, unable to make `read` blocking: {}",
                err
            );
        }

        run(app.clone(), &mut stream);

        let mut byte = [0u8; 1];

        loop {
            match stream.peek(&mut byte) {
                Ok(_bytes) => {
                    run(app.clone(), &mut stream);
                }
                Err(err) if err.kind() == io::ErrorKind::ConnectionAborted => break,
                Err(err) => {
                    log::error!("{}", err)
                }
            }
        }
    }

    fn thread_handle(app: Arc<BuiltApp>, stream: &mut TcpStream) -> Result<(), ThreadError> {
        let (header_data, body) = http::read_request(stream)?;

        let (service, params) = app
            .tree
            .get(&header_data.method)
            .and_then(|tree| tree.find(&header_data.url))
            .map(|(service, params)| {
                let mut map: ArrayMap<String, String, 32> = ArrayMap::new();

                for (key, value) in params.into_iter() {
                    map.insert(key.to_string(), value.to_string());
                }

                (Arc::clone(service), map)
            })
            .unwrap_or_else(|| (app.default_service.clone(), ArrayMap::new()));

        let compress = if let Some(header) = header_data.headers.get(&ACCEPT_ENCODING) {
            header.contains("deflate")
        } else {
            false
        };

        let mut request = HttpRequest {
            header_data,
            body,
            params,
            data: Arc::clone(&app.data),
            extensions: Extensions::new(),
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
