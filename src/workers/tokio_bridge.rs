use tracing::{instrument, info, trace};
use stakker::*;


pub struct Sender<T> {
    tx: flume::Sender<T>,
    waker: Waker,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), flume::SendError<T>> {
        self.tx.send(value)?;
        self.waker.wake();
        Ok(())
    }
}


#[derive(Clone)]
pub struct TokioHandle {
    handle: tokio::runtime::Handle,
}

impl TokioHandle {
    pub fn new(handle: tokio::runtime::Handle) -> Self {
        TokioHandle {
            handle,
        }
    }

    #[instrument(skip_all)]
    pub fn spawn_task<T, Fut>(
        &mut self,
        fwd: Fwd<T>,
        core: &mut Core,
        run: impl FnOnce(Sender<T>) -> Fut + Send + 'static,
    )
    where
        T: Send,
        Fut: std::future::Future + Send + 'static,
        Fut::Output: Send + 'static,
    {
        let (tx, rx) = flume::unbounded();

        let waker = core.waker(move |_stakker, _deleted| {
            // to allow for the possibility of buffering messages
            // from the async task by delaying the wakeup call we
            // drain the message sink here to make sure that all
            // messages get pushed to the fwd callback at once
            for message in rx.drain() {
                fwd!([fwd], message);
            }
        });

        // TODO: store the JoinHandle and implement Drop
        // so that async tasks get aborted when TokioHandle is dropped
        self.handle.spawn(async move {
            run(Sender { tx, waker }).await;
        });
    }

    pub fn spawn_ret<T, Fut>(
        &mut self,
        ret: Ret<T>,
        core: &mut Core,
        run: impl FnOnce() -> Fut + Send + 'static,
    )
    where
        T: Send,
        Fut: std::future::Future<Output = T> + Send + 'static,
    {
        let (tx, rx) = flume::unbounded();

        let mut ret_option = Some(ret);
        let waker = core.waker(move |_stakker, _deleted| {
            if let Some(ret) = ret_option.take() {
                // TODO: handle receiver errors gracefully
                let result = rx.recv().unwrap();
                ret!([ret], result);
            }
        });

        // TODO: store the JoinHandle and implement Drop
        // so that async tasks get aborted when TokioHandle is dropped
        self.handle.spawn(async move {
            let result = run().await;
            tx.send(result).unwrap();
            waker.wake();
        });
    }
}


#[derive(Debug)]
pub enum TokioCommand {
    Shutdown,
}

pub struct TokioRuntime {
    tx: flume::Sender<TokioCommand>,
    thread: std::thread::JoinHandle<()>,
    handle: tokio::runtime::Handle,
}

impl TokioRuntime {
    pub fn new() -> Self {
        let (tx, rx) = flume::unbounded();
        let (thread, handle) = Self::spawn_runtime(rx);

        TokioRuntime {
            tx,
            thread,
            handle,
        }
    }

    #[instrument(skip_all)]
    fn spawn_runtime(rx: flume::Receiver<TokioCommand>) -> (std::thread::JoinHandle<()>, tokio::runtime::Handle) {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let handle = rt.handle().to_owned();

        let thread = std::thread::spawn(move || {
            info!("Starting tokio runtime");
            let _guard = rt.enter();

            while let Ok(command) = rx.recv() {
                trace!(?command, "Received command");
                match command {
                    TokioCommand::Shutdown => {
                        info!("Shutting down");
                        break
                    }
                }
            }
        });

        (thread, handle)
    }

    pub fn handle(&mut self) -> TokioHandle {
        TokioHandle::new(self.handle.clone())
    }

    pub fn shutdown(self) -> std::thread::JoinHandle<()> {
        self.tx.send(TokioCommand::Shutdown).unwrap();
        self.thread
    }
}
