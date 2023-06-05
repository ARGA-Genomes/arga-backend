use std::time::Duration;

use stakker::*;
use tracing::{info, instrument};

use crate::database::schema;
use crate::database::models::{Job, JobStatus};

use super::taxa_importer::TaxaImporter;
use super::conservation_status_importer::ConservationStatusImporter;
use super::specimen_importer::SpecimenImporter;
use super::tokio_bridge::TokioHandle;


pub struct Manager {
    interval: Duration,
    _store: ActorOwn<PostgresStore>,
    poller: ActorOwn<JobPoller>,
    allocator: ActorOwn<Allocator>,
}

impl Manager {
    pub fn init(cx: CX![], interval: Duration) -> Option<Self> {
        let store = actor!(cx, PostgresStore::init(), ret_nop!());
        let poller = actor!(cx, JobPoller::init(store.clone()), ret_nop!());
        let allocator = actor!(cx, Allocator::init(store.clone()), ret_nop!());

        call!([cx], poll());

        Some(Self {
            interval,
            _store: store,
            poller,
            allocator,
        })
    }

    pub fn poll(&self, cx: CX![]) {
        let ret = ret_some_to!([self.allocator], recv_job() as (Option<Job>));
        call!([self.poller], next_job(ret));

        // call poll again after the defined interval
        after!(self.interval, [cx], poll());
    }
}


use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;


type PgPool = Pool<AsyncPgConnection>;

pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    pub fn init(cx: CX![]) -> Option<Self> {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let mut handle = cx.anymap_get::<TokioHandle>();

        handle.spawn_ret(ret_to!([cx], Self::setup_connection() as (PgPool)), cx, || async move {
            info!("Connecting to database");

            let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url);
            let pool = Pool::builder().build(config).await.unwrap();

            info!("Connected");
            pool
        });

        None
    }

    fn setup_connection(_cx: CX![], pool: Option<PgPool>) -> Option<Self> {
        Some(Self {
            pool: pool.unwrap(),
        })
    }

    fn pool(&self, _cx: CX![], ret: Ret<PgPool>) {
        ret!([ret], self.pool.clone());
    }
}


pub struct JobPoller {
    handle: TokioHandle,
    pool: Pool<AsyncPgConnection>,
}

impl JobPoller {
    pub fn init(cx: CX![], postgres: Actor<PostgresStore>) -> Option<Self> {
        let ret = ret_to!([cx], Self::setup_connection() as (PgPool));
        call!([postgres], pool(ret));
        None
    }

    fn setup_connection(cx: CX![], pool: Option<PgPool>) -> Option<Self> {
        let handle = cx.anymap_get::<TokioHandle>();
        Some(Self {
            handle,
            pool: pool.unwrap(),
        })
    }

    pub fn next_job(&mut self, cx: CX![], ret: Ret<Option<Job>>) {
        let pool = self.pool.clone();

        self.handle.spawn_ret(ret, cx, || async move {
            use schema::jobs::dsl::*;
            let mut conn = pool.get().await.unwrap();

            jobs
                .filter(status.eq(JobStatus::Pending))
                .first(&mut conn)
                .await
                .optional()
                .unwrap()
        });
    }
}


pub struct Allocator {
    handle: TokioHandle,
    pool: Pool<AsyncPgConnection>,

    taxa_importer: ActorOwn<TaxaImporter>,
    conservation_status_importer: ActorOwn<ConservationStatusImporter>,
    specimen_importer: ActorOwn<SpecimenImporter>,
}

impl Allocator {
    pub fn init(cx: CX![], postgres: Actor<PostgresStore>) -> Option<Self> {
        let ret = ret_to!([cx], Self::setup_connection() as (PgPool));
        call!([postgres], pool(ret));
        None
    }

    fn setup_connection(cx: CX![], pool: Option<PgPool>) -> Option<Self> {
        let handle = cx.anymap_get::<TokioHandle>();
        Some(Self {
            handle,
            pool: pool.unwrap(),

            taxa_importer: actor!(cx, TaxaImporter::init(), ret_nop!()),
            conservation_status_importer: actor!(cx, ConservationStatusImporter::init(), ret_nop!()),
            specimen_importer: actor!(cx, SpecimenImporter::init(), ret_nop!()),
        })
    }

    #[instrument(skip(self, cx))]
    pub fn recv_job(&mut self, cx: CX![], job: Option<Job>) {
        if let Some(job) = job {
            info!("Taking job");
            let pool = self.pool.clone();

            let ret = match job.worker.as_str() {
                "import_csv" => ret_some_to!([self.taxa_importer], import() as (Job)),
                "import_conservation_status" => ret_some_to!([self.conservation_status_importer], import() as (Job)),
                "import_specimen" => ret_some_to!([self.specimen_importer], import() as (Job)),
                _ => panic!("Unknown job worker: {}", job.worker)
            };

            // update the status so that it is only allocated to one worker
            self.handle.spawn_ret(ret, cx, move || async move {
                use schema::jobs::dsl::*;
                let mut conn = pool.get().await.unwrap();

                diesel::update(jobs)
                    .filter(id.eq(job.id))
                    .set(status.eq(JobStatus::Initialized))
                    .get_result(&mut conn)
                    .await
                    .unwrap()
            });
        }
    }
}