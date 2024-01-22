mod error;
mod extractors;
mod matchers;
mod importers;

mod manager;
mod threaded_job;

use std::time::{Instant, Duration};

use stakker::*;
use tracing::info;

use self::manager::Manager;


fn main() {
    dotenvy::dotenv().ok();
    run();
}


fn run() {
    tracing_subscriber::fmt::init();

    let mut stakker0 = Stakker::new(Instant::now());
    let stakker = &mut stakker0;

    // allows async threads to wake the actor thread
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let waker = move || { tx.send(0).expect("The waker receiver has been dropped") };
    stakker.set_poll_waker(waker);

    // create the manager actor which spawns other actors
    // to process jobs
    let _manager = actor!(stakker, Manager::init(Duration::from_secs(1)), ret_shutdown!(stakker));

    // worker main loop
    info!("Starting actor main loop");
    stakker.run(Instant::now(), false);

    while stakker.not_shutdown() {
        let max_wait = stakker.next_wait_max(Instant::now(), Duration::from_secs(60), false);

        match rx.recv_timeout(max_wait) {
            Ok(_) => stakker.poll_wake(),
            Err(err) => {
                match err {
                    std::sync::mpsc::RecvTimeoutError::Timeout => {},
                    std::sync::mpsc::RecvTimeoutError::Disconnected => stakker.shutdown(StopCause::Lost),
                }
            },
        }

        stakker.run(Instant::now(), false);
    }
}
