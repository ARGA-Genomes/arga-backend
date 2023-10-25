use std::time::Duration;

use stakker::*;
use tracing::{info, instrument};
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};

use arga_core::schema;
use arga_core::models::{Job, JobStatus};

use super::threaded_job::ThreadedJob;


type PgPool = Pool<ConnectionManager<PgConnection>>;


pub struct Manager {
    interval: Duration,
    poller: ActorOwn<JobPoller>,
    allocator: ActorOwn<Allocator>,
}

impl Manager {
    pub fn init(cx: CX![], interval: Duration) -> Option<Self> {
        let url = arga_core::get_database_url();
        let manager = ConnectionManager::<PgConnection>::new(url);
        let pool = Pool::builder().build(manager).expect("Could not build connection pool");

        let poller = actor!(cx, JobPoller::init(pool.clone()), ret_nop!());
        let allocator = actor!(cx, Allocator::init(pool.clone()), ret_nop!());

        call!([cx], poll());

        Some(Self {
            interval,
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


pub struct JobPoller {
    pool: PgPool,
}

impl JobPoller {
    pub fn init(_cx: CX![], pool: PgPool) -> Option<Self> {
        Some(Self {
            pool,
        })
    }

    pub fn next_job(&mut self, _cx: CX![], ret: Ret<Option<Job>>) {
        use schema::jobs::dsl::*;

        let mut conn = self.pool.get().expect("Failed to get a connection");
        let job = jobs
            .filter(status.eq(JobStatus::Pending))
            .order(created_at.asc())
            .first(&mut conn)
            .optional()
            .expect("Could not query for the next job");

        ret!([ret], job);
    }
}


pub struct Allocator {
    pool: PgPool,
    threaded_job: ActorOwn<ThreadedJob>,
}

impl Allocator {
    pub fn init(cx: CX![], pool: PgPool) -> Option<Self> {
        Some(Self {
            pool,
            threaded_job: actor!(cx, ThreadedJob::init(), ret_nop!()),
        })
    }

    #[instrument(skip(self, _cx))]
    pub fn recv_job(&mut self, _cx: CX![], job: Option<Job>) {
        use schema::jobs::dsl::*;

        if let Some(job) = job {
            info!("Taking job");
            let pool = self.pool.clone();

            // update the status so that it is only allocated to one worker
            let mut conn = pool.get().expect("Failed to get a connection");
            let job = diesel::update(jobs)
                .filter(id.eq(job.id))
                .set(status.eq(JobStatus::Initialized))
                .get_result(&mut conn)
                .expect("Could not get a lock on the job");

            call!([self.threaded_job], run(job));
        }
    }
}
