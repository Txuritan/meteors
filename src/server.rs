use {
    crate::{
        app::BuiltApp, handler::HandlerError, http::HttpError, server::pool::ThreadPool,
        service::Service, App, HttpRequest,
    },
    std::{
        collections::BTreeMap,
        fmt,
        io::{self, Read as _},
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

impl From<std::io::Error> for RunError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<ctrlc::Error> for RunError {
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

        let (pool, sender) = ThreadPool::new(4, Arc::clone(&self.close), |data| {
            if let Err(err) = Self::thread_handle(data) {
                log::error!("unable to handle thread");

                match err {
                    ThreadError::Handler(err) => log::error!("route handler error: {:?}", err),
                    ThreadError::Http(err) => log::error!("invalid http: {:?}", err),
                    ThreadError::Io(err) => log::error!("{}", err),
                    ThreadError::ParseInt(err) => log::error!("{}", err),
                    ThreadError::Utf8(err) => log::error!("{}", err),
                }
            }
        });

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
    Handler(HandlerError),
    Http(HttpError),
    Io(io::Error),
    ParseInt(std::num::ParseIntError),
    Utf8(std::string::FromUtf8Error),
}

impl From<HandlerError> for ThreadError {
    fn from(err: HandlerError) -> Self {
        ThreadError::Handler(err)
    }
}

impl From<HttpError> for ThreadError {
    fn from(err: HttpError) -> Self {
        ThreadError::Http(err)
    }
}

impl From<io::Error> for ThreadError {
    fn from(err: io::Error) -> Self {
        ThreadError::Io(err)
    }
}

impl From<std::num::ParseIntError> for ThreadError {
    fn from(v: std::num::ParseIntError) -> Self {
        Self::ParseInt(v)
    }
}

impl From<std::string::FromUtf8Error> for ThreadError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        ThreadError::Utf8(err)
    }
}

impl HttpServer<SocketAddr> {
    fn thread_handle(
        (app, mut stream, _addr): (Arc<BuiltApp>, TcpStream, SocketAddr),
    ) -> Result<(), ThreadError> {
        struct State {
            data: Vec<u8>,
            total_read: usize,
            amount_read: usize,
            read_buffer: [u8; BUFFER_SIZE],
        }

        fn double_newline(bytes: &[u8]) -> bool {
            bytes == &b"\r\n\r\n"[..]
        }

        const BUFFER_SIZE: usize = 512;
        const MAX_BYTES: usize = 1028 * 8;

        let mut state = State {
            data: Vec::with_capacity(512),
            total_read: 0,
            amount_read: BUFFER_SIZE,
            read_buffer: [0; BUFFER_SIZE],
        };

        loop {
            if state.amount_read == 0 {
                break;
            }

            state.amount_read = stream.read(&mut state.read_buffer)?;

            if state.amount_read == 0 {
                break;
            }

            (state.total_read) += state.amount_read;

            state
                .data
                .extend_from_slice(&state.read_buffer[..state.amount_read]);

            (state.read_buffer) = [0; BUFFER_SIZE];

            if state.data.windows(4).any(double_newline) {
                break;
            }

            if state.total_read >= MAX_BYTES {
                break;
            }
        }

        if let Some(i) = state.data.windows(4).position(double_newline) {
            let header_bytes = &state.data[..(i + 2)];
            let header_str = String::from_utf8_lossy(header_bytes);

            let header_data = HttpRequest::parse_header(header_str.as_ref())?;

            let (header_data, body) =
                if let Some(header) = header_data.headers.get("Content-Length") {
                    let amount_of_bytes = header.trim().parse::<u64>()?;

                    let mut body = Vec::with_capacity(amount_of_bytes as usize);

                    stream
                        .by_ref()
                        .take(amount_of_bytes)
                        .read_to_end(&mut body)?;

                    (header_data, Vec::from(&body[..]))
                } else {
                    (header_data, vec![])
                };

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
                            .collect::<BTreeMap<_, _>>(),
                    )
                })
                .unwrap_or_else(|| (app.not_found.clone(), BTreeMap::new()));

            let mut request =
                HttpRequest::from_parts(header_data, body, parameters, Arc::clone(&app.data));

            for middleware in &*app.middleware {
                middleware.before(&mut request);
            }

            let response = service.call(&mut request)?;

            for middleware in &*app.middleware {
                middleware.after(&request, &response);
            }

            response.into_stream(&mut stream)?;

            Ok(())
        } else {
            Err(HttpError::InvalidRequest.into())
        }
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
