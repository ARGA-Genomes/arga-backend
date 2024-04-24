pub mod nomenclatural_acts;
pub mod specimens;

use std::path::PathBuf;

use arga_core::models::DatasetVersion;
use arga_core::schema;
use chrono::{DateTime, Utc};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::*;
use uuid::Uuid;

use super::Error;

type PgPool = Pool<ConnectionManager<PgConnection>>;

#[derive(clap::Subcommand)]
pub enum Command {
    /// Process and import a csv as operation logs
    #[command(subcommand)]
    Import(ImportCommand),

    /// Reduce operation logs into entity objects
    #[command(subcommand)]
    Reduce(ReduceCommand),
}

#[derive(clap::Subcommand)]
pub enum ImportCommand {
    /// Extract nomenclatural acts from a CSV dataset
    NomenclaturalActs {
        dataset_id: String,
        version: String,
        created_at: String,
        path: PathBuf,
    },

    /// Extract specimens from a CSV dataset
    Specimens {
        dataset_id: String,
        version: String,
        created_at: String,
        path: PathBuf,
    },
}

#[derive(clap::Subcommand)]
pub enum ReduceCommand {
    NomenclaturalActs,
    Specimens,
    CollectionEvents,
}

pub fn process_command(cmd: &Command) {
    match cmd {
        Command::Import(cmd) => match cmd {
            ImportCommand::NomenclaturalActs {
                dataset_id,
                version,
                created_at,
                path,
            } => nomenclatural_acts::process(
                path.clone(),
                create_dataset_version(dataset_id, version, created_at).unwrap(),
            )
            .unwrap(),
            ImportCommand::Specimens {
                dataset_id,
                version,
                created_at,
                path,
            } => specimens::process(path.clone(), create_dataset_version(dataset_id, version, created_at).unwrap())
                .unwrap(),
        },
        Command::Reduce(cmd) => match cmd {
            ReduceCommand::NomenclaturalActs => nomenclatural_acts::reduce().unwrap(),
            ReduceCommand::Specimens => specimens::reduce_specimens().unwrap(),
            ReduceCommand::CollectionEvents => specimens::reduce_collections().unwrap(),
        },
    }
}

pub fn get_pool() -> Result<PgPool, Error> {
    let url = arga_core::get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    let pool = Pool::builder().build(manager)?;
    Ok(pool)
}

fn create_dataset_version(dataset_id: &str, version: &str, created_at: &str) -> Result<DatasetVersion, Error> {
    use schema::dataset_versions;

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    let dataset_version = diesel::insert_into(dataset_versions::table)
        .values(DatasetVersion {
            id: Uuid::new_v4(),
            dataset_id: find_database_id(&dataset_id)?,
            version: version.to_string(),
            created_at: DateTime::parse_from_rfc3339(&created_at).unwrap().to_utc(),
            imported_at: Utc::now(),
        })
        .returning(DatasetVersion::as_select())
        .get_result(&mut conn)?;

    Ok(dataset_version)
}

fn find_database_id(dataset_id: &str) -> Result<Uuid, Error> {
    use schema::datasets::dsl::*;

    let pool = get_pool()?;
    let mut conn = pool.get()?;

    let uuid = datasets
        .filter(global_id.eq(dataset_id))
        .select(id)
        .get_result::<Uuid>(&mut conn)?;
    Ok(uuid)
}
