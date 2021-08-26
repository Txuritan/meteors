use {
    crate::{
        app::BuiltApp,
        http::{self, headers::ACCEPT_ENCODING},
        server::pool::ThreadPool,
        service::Service,
        App, Error, HttpRequest,
    },
    std::{
        collections::HashMap,
        fmt,
        io::{self},
        net::{SocketAddr, TcpListener, TcpStream},
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        thread::{self, JoinHandle},
    },
};

#[derive(Debug)]
pub enum RunError {
    Io(std::io::Error),
    Signal(ctrlc::Error),
}

impl const From<std::io::Error> for RunError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl const From<ctrlc::Error> for RunError {
    fn from(err: ctrlc::Error) -> Self {
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
        ctrlc::set_handler({
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
        let (header_data, body) = HttpRequest::parse_reader(stream)?;

        let (service, parameters) = app
            .tree
            .get(&header_data.method)
            .and_then(|tree| tree.find(&header_data.url))
            .map(|(service, parameters)| {
                (
                    Arc::clone(service),
                    parameters
                        .into_iter()
                        .map(|(key, value)| (key.to_string(), value.to_string()))
                        .collect::<HashMap<_, _>>(),
                )
            })
            .unwrap_or_else(|| (app.default_service.clone(), HashMap::new()));

        let compress = if let Some(header) = header_data.headers.get(&ACCEPT_ENCODING) {
            header.contains("deflate")
        } else {
            false
        };

        let mut request =
            HttpRequest::from_parts(header_data, body, parameters, Arc::clone(&app.data));

        for middleware in &*app.middleware {
            middleware.before(&mut request);
        }

        let response = service.call(&mut request)?;

        for middleware in &*app.middleware {
            middleware.after(&request, &response);
        }

        response.into_stream(compress, stream)?;

        Ok(())
    }
}

mod pool {
    use std::{
        marker::PhantomData,
        sync::{
            atomic::{AtomicBool, Ordering},
            mpsc::{self, Receiver, RecvTimeoutError, Sender},
            Arc, Mutex,
        },
        thread::{self, JoinHandle},
        time::Duration,
    };

    pub struct ThreadPool<Data>
    where
        Data: Send + Sync + 'static,
    {
        workers: Vec<Worker<Data>>,
    }

    impl<Data> ThreadPool<Data>
    where
        Data: Send + Sync + 'static,
    {
        pub fn new<F>(size: usize, close: Arc<AtomicBool>, handler: F) -> (Self, Sender<Data>)
        where
            F: Fn(Data) + Clone + Send + Sync + 'static,
        {
            let (sender, receiver) = mpsc::channel();

            let receiver = Arc::new(Mutex::new(receiver));

            let workers = (0..size)
                .into_iter()
                .map(|id| {
                    Worker::new(
                        id,
                        Arc::clone(&close),
                        Arc::clone(&receiver),
                        handler.clone(),
                    )
                })
                .collect();

            (Self { workers }, sender)
        }

        pub fn join(self) {
            for worker in self.workers {
                worker.join()
            }
        }
    }

    struct Worker<Data>
    where
        Data: Send + Sync + 'static,
    {
        id: usize,
        thread: JoinHandle<()>,
        _data: PhantomData<Data>,
    }

    impl<Data> Worker<Data>
    where
        Data: Send + Sync + 'static,
    {
        fn new<F>(
            id: usize,
            close: Arc<AtomicBool>,
            receiver: Arc<Mutex<Receiver<Data>>>,
            handle: F,
        ) -> Self
        where
            F: Fn(Data) + Clone + Send + Sync + 'static,
        {
            let thread = thread::spawn(move || Self::inner(id, close, receiver, handle));

            Self {
                id,
                thread,
                _data: PhantomData,
            }
        }

        fn inner<F>(
            id: usize,
            close: Arc<AtomicBool>,
            receiver: Arc<Mutex<Receiver<Data>>>,
            handle: F,
        ) where
            F: Fn(Data) + Clone + Send + Sync + 'static,
        {
            loop {
                let received = {
                    let receiver = receiver.lock().unwrap();

                    receiver.recv_timeout(Duration::from_millis(100))
                };

                match received {
                    Ok(data) => {
                        log::trace!("worker {} received a request", id);

                        handle(data)
                    }
                    Err(RecvTimeoutError::Disconnected) => break,
                    Err(RecvTimeoutError::Timeout) => {
                        if close.load(Ordering::SeqCst) {
                            break;
                        }
                    }
                }
            }
        }

        fn join(self) {
            self.thread.join().unwrap();

            log::trace!("shutdown worker {}", self.id);
        }
    }
}
