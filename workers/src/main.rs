mod error;
mod extractors;
mod matchers;
mod importers;

mod manager;
mod threaded_job;

mod tokio_bridge;
mod taxa_importer;
mod synonym_importer;
mod conservation_status_importer;
mod specimen_importer;
mod marker_importer;
mod vernacular_importer;

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

    // store the tokio handle instance in the anymap so that
    // actors can execute async tasks without needing to thread
    // through handle clones down the tree. since the tokio runtime
    // is used as part of the main loop we consider it part of the
    // environment and expect actors to gracefully handle errors
    // that occur when trying to spawn with a failed tokio runtime
    let mut runtime = tokio_bridge::TokioRuntime::new();
    stakker.anymap_set(runtime.handle());

    // create the manager actor which spawns other actors
    // to process jobs
    let _manager = actor!(stakker, Manager::init(Duration::from_secs(10)), ret_shutdown!(stakker));

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

    // gracefully shutdown the tokio runtime thread
    runtime.shutdown().join().unwrap();
}
