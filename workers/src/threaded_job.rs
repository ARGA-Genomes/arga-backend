use std::path::Path;

use arga_core::models::{Dataset, Job};
use arga_core::schema;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use serde::Deserialize;
use stakker::*;
use tracing::{error, info};

use super::error::Error;
use super::importers::{
    accession_importer,
    admin_media_importer,
    annotation_importer,
    assembly_importer,
    classification_importer,
    collection_importer,
    dataset_importer,
    deposition_importer,
    dna_extraction_importer,
    name_attribute_importer,
    name_importer,
    nomenclatural_act_importer,
    region_importer,
    sequence_importer,
    source_importer,
    subsample_importer,
    taxon_photo_importer,
    vernacular_importer,
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
            },
        );

        Some(Self { thread })
    }

    pub fn run(&mut self, _cx: CX![], job: Job) {
        self.thread.send(job);
    }

    fn recv(&mut self, _cx: CX![], _job: Job) {}

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

        // dataset isn't used for all imports like the dataset importer itself
        // so we have to make sure to keep it optional
        let dataset = Self::get_dataset(&data.dataset, pool);

        let mut context = vec![];
        if let Ok(dataset) = dataset.as_ref().map(|d| d.clone()) {
            context.push(dataset);
        }
        for name in &data.isolation_context {
            context.push(Self::get_dataset(&name, pool)?);
        }

        match worker {
            "import_source" => source_importer::import(path, pool)?,
            "import_dataset" => dataset_importer::import(path, pool)?,
            "import_name" => name_importer::import(path, pool)?,
            "import_taxon_photo" => taxon_photo_importer::import(path, pool)?,
            "import_vernacular" => vernacular_importer::import(path, &dataset?, pool)?,
            "import_region" => region_importer::import(path, pool)?,
            "import_collection" => collection_importer::import(path, &dataset?, &context, pool)?,
            "import_accession" => accession_importer::import(path, &dataset?, &context, pool)?,
            "import_subsample" => subsample_importer::import(path, &dataset?, &context, pool)?,
            "import_dna_extraction" => dna_extraction_importer::import(path, &dataset?, &context, pool)?,
            "import_sequence" => sequence_importer::import(path, &dataset?, &context, pool)?,
            "import_assembly" => assembly_importer::import(path, &dataset?, pool)?,
            "import_annotation" => annotation_importer::import(path, &dataset?, pool)?,
            "import_deposition" => deposition_importer::import(path, &dataset?, pool)?,
            "import_name_attribute" => name_attribute_importer::import(path, pool)?,
            "import_admin_media" => admin_media_importer::import(path, data.dataset.clone(), pool)?,

            "import_classification" => classification_importer::import(path, pool)?,
            "import_nomenclatural_act" => nomenclatural_act_importer::import(path, pool)?,
            _ => panic!("Unknown job worker: {}", worker),
        }

        Ok(())
    }

    fn get_dataset(dataset_global_id: &str, pool: &mut PgPool) -> Result<Dataset, Error> {
        use schema::datasets::dsl::*;
        let mut conn = pool.get()?;
        let dataset = datasets
            .filter(global_id.eq(dataset_global_id))
            .get_result::<Dataset>(&mut conn)
            .map_err(|e| e.into());

        dataset
    }
}
