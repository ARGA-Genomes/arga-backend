use std::path::Path;

use serde::Deserialize;
use stakker::*;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use tracing::{info, error};

use arga_core::models::Job;

use super::error::Error;
use super::importers::{
    source_importer,
    dataset_importer,
    collection_importer,
    accession_importer,
    subsample_importer,
    dna_extraction_importer,
    sequence_importer,
    taxon_importer,
    synonym_importer,
    vernacular_importer,
    region_importer,
    ecology_importer,
    conservation_status_importer,
    indigenous_knowledge_importer,
};


type PgPool = Pool<ConnectionManager<PgConnection>>;


#[derive(Debug, Deserialize)]
struct ImportJobData {
    name: String,
    description: Option<String>,
    url: Option<String>,
    tmp_name: String,
}

pub struct ThreadedJob {
    thread: PipedThread<Job, Job>,
}

impl ThreadedJob {
    pub fn init(cx: CX![]) -> Option<Self> {
        let thread = PipedThread::spawn(
            fwd_to!([cx], recv() as (Job)),
            fwd_to!([cx], term() as (Option<String>)),
            cx,
            move |link| {
                while let Some(job) = link.recv() {
                    Self::process(job);
                }
            }
        );

        Some(Self {
            thread,
        })
    }

    pub fn run(&mut self, _cx: CX![], job: Job) {
        self.thread.send(job);
    }

    fn recv(&mut self, _cx: CX![], _job: Job) {

    }

    fn term(&self, cx: CX![], panic: Option<String>) {
        if let Some(msg) = panic {
            panic!("Unexpected thread failure: {}", msg);
        }
        cx.stop();
    }

    fn process(job: Job) {
        info!("Running threaded job");

        let url = arga_core::get_database_url();
        let manager = ConnectionManager::<PgConnection>::new(url);
        let mut pool = Pool::builder().build(manager).expect("Could not build connection pool");

        if let Some(payload) = job.payload {
            match serde_json::from_value::<ImportJobData>(payload) {
                Ok(data) => Self::run_worker(&job.worker, &data, &mut pool).unwrap(),
                Err(err) => error!(?err, "Invalid JSON payload"),
            }
        }
    }

    fn run_worker(worker: &str, data: &ImportJobData, pool: &mut PgPool) -> Result<(), Error> {
        let tmp_path = std::env::var("ADMIN_TMP_UPLOAD_STORAGE").expect("No upload storage specified");
        let path = Path::new(&tmp_path).join(&data.tmp_name);

        match worker {
            "import_source" => source_importer::import(path, pool)?,
            "import_dataset" => dataset_importer::import(path, pool)?,
            "import_taxon" => taxon_importer::import(path, pool)?,
            "import_synonym" => synonym_importer::import(path, pool)?,
            "import_vernacular" => vernacular_importer::import(path, pool)?,
            "import_region" => region_importer::import(path, pool)?,
            "import_ecology" => ecology_importer::import(path, pool)?,
            "import_conservation_status" => {
                info!(name=data.name, "Importing conservation status");
                let source = conservation_status_importer::get_or_create_dataset(&data.name, &data.description, pool)?;
                conservation_status_importer::import(path, &source, pool)?;
            }
            "import_indigenous_knowledge" => indigenous_knowledge_importer::import(path, pool)?,
            "import_collection" => collection_importer::import(path, &data.name, pool)?,
            "import_accession" => accession_importer::import(path, &data.name, pool)?,
            "import_subsample" => subsample_importer::import(path, &data.name, pool)?,
            "import_dna_extraction" => dna_extraction_importer::import(path, &data.name, pool)?,
            "import_sequence" => {
                info!(name=data.name, "Importing sequence events");
                let source = sequence_importer::get_or_create_dataset(&data.name, &data.description, pool)?;
                sequence_importer::import(path, &source, pool)?;
            }
            _ => {}
        }

        Ok(())
    }
}
