use std::path::Path;

use arga_core::schema;
use serde::Deserialize;
use stakker::*;
use diesel::*;
use diesel::r2d2::{Pool, ConnectionManager};
use tracing::{info, error};

use arga_core::models::{Job, Dataset};

use super::error::Error;
use super::importers::{
    source_importer,
    dataset_importer,
    collection_importer,
    accession_importer,
    subsample_importer,
    dna_extraction_importer,
    sequence_importer,
    assembly_importer,
    annotation_importer,
    deposition_importer,
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
    dataset: String,
    isolation_context: Vec<String>,
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

        let dataset = Self::get_dataset(&data.dataset, pool)?;

        let mut context = vec![dataset.clone()];
        for name in &data.isolation_context {
            context.push(Self::get_dataset(&name, pool)?);
        }

        match worker {
            "import_source" => source_importer::import(path, pool)?,
            "import_dataset" => dataset_importer::import(path, pool)?,
            "import_taxon" => taxon_importer::import(path, &dataset, pool)?,
            "import_synonym" => synonym_importer::import(path, &dataset, pool)?,
            "import_vernacular" => vernacular_importer::import(path, pool)?,
            "import_region" => region_importer::import(path, pool)?,
            "import_ecology" => ecology_importer::import(path, pool)?,
            "import_conservation_status" => {
                // info!(name=data.name, "Importing conservation status");
                // let source = conservation_status_importer::get_or_create_dataset(&data.name, &data.description, pool)?;
                // conservation_status_importer::import(path, &source, pool)?;
            }
            "import_indigenous_knowledge" => indigenous_knowledge_importer::import(path, pool)?,
            "import_collection" => collection_importer::import(path, &dataset, pool)?,
            "import_accession" => accession_importer::import(path, &dataset, pool)?,
            "import_subsample" => subsample_importer::import(path, &dataset, pool)?,
            "import_dna_extraction" => dna_extraction_importer::import(path, &dataset, pool)?,
            "import_sequence" => sequence_importer::import(path, &dataset, &context, pool)?,
            "import_assembly" => assembly_importer::import(path, &dataset, pool)?,
            "import_annotation" => annotation_importer::import(path, &dataset, pool)?,
            "import_deposition" => deposition_importer::import(path, &dataset, pool)?,
            _ => {}
        }

        Ok(())
    }

    fn get_dataset(dataset_global_id: &str, pool: &mut PgPool) -> Result<Dataset, Error> {
        use schema::datasets::dsl::*;
        let mut conn = pool.get()?;
        let dataset = datasets
            .filter(global_id.eq(dataset_global_id))
            .get_result::<Dataset>(&mut conn)?;

        Ok(dataset)
    }
}
