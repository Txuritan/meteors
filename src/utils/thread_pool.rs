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

    fn inner<F>(id: usize, close: Arc<AtomicBool>, receiver: Arc<Mutex<Receiver<Data>>>, handle: F)
    where
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
