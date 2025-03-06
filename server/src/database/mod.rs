pub mod extensions;

pub mod datasets;
pub mod dna_extracts;
pub mod list_groups;
pub mod maps;
pub mod markers;
pub mod names;
pub mod overview;
pub mod provenance;
pub mod sequences;
pub mod sources;
pub mod species;
pub mod specimens;
pub mod stats;
pub mod subsamples;
pub mod taxa;


pub use arga_core::{get_database_url, models, schema, schema_gnl};
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use thiserror::Error;

use self::extensions::pagination::Page;
use crate::http::Error as HttpError;


pub type PgPool = Pool<AsyncPgConnection>;


#[derive(Error, Debug)]
pub enum Error {
    #[error("the record '{0}' could not found")]
    NotFound(String),

    #[error(transparent)]
    Connection(#[from] diesel::result::Error),

    #[error(transparent)]
    Pool(#[from] diesel_async::pooled_connection::PoolError),

    #[error(transparent)]
    GetPool(#[from] diesel_async::pooled_connection::bb8::RunError),
}

impl From<diesel::result::Error> for HttpError {
    fn from(err: diesel::result::Error) -> Self {
        HttpError::Database(Error::Connection(err))
    }
}

impl From<diesel_async::pooled_connection::PoolError> for HttpError {
    fn from(err: diesel_async::pooled_connection::PoolError) -> Self {
        HttpError::Database(Error::Pool(err))
    }
}

impl From<diesel_async::pooled_connection::bb8::RunError> for HttpError {
    fn from(err: diesel_async::pooled_connection::bb8::RunError) -> Self {
        HttpError::Database(Error::GetPool(err))
    }
}

pub type PageResult<T> = Result<Page<T>, Error>;

#[derive(Clone)]
pub struct Database {
    pub pool: Pool<AsyncPgConnection>,

    pub names: names::NameProvider,
    pub markers: markers::MarkerProvider,
    pub overview: overview::OverviewProvider,
    pub stats: stats::StatsProvider,
    pub species: species::SpeciesProvider,
    pub sources: sources::SourceProvider,
    pub datasets: datasets::DatasetProvider,
    pub taxa: taxa::TaxaProvider,
    pub specimens: specimens::SpecimenProvider,
    pub subsamples: subsamples::SubsampleProvider,
    pub dna_extracts: dna_extracts::DnaExtractProvider,
    pub list_groups: list_groups::ListGroupProvider,
    pub sequences: sequences::SequenceProvider,
    pub maps: maps::MapsProvider,
    pub provenance: provenance::ProvenanceProvider,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Database, Error> {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url);
        // let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_setup(url, establish_tls_connection);
        let pool = Pool::builder().build(config).await?;

        Ok(Database {
            names: names::NameProvider { pool: pool.clone() },
            markers: markers::MarkerProvider { pool: pool.clone() },
            overview: overview::OverviewProvider { pool: pool.clone() },
            stats: stats::StatsProvider { pool: pool.clone() },
            species: species::SpeciesProvider { pool: pool.clone() },
            sources: sources::SourceProvider { pool: pool.clone() },
            datasets: datasets::DatasetProvider { pool: pool.clone() },
            taxa: taxa::TaxaProvider { pool: pool.clone() },
            specimens: specimens::SpecimenProvider { pool: pool.clone() },
            subsamples: subsamples::SubsampleProvider { pool: pool.clone() },
            dna_extracts: dna_extracts::DnaExtractProvider { pool: pool.clone() },
            list_groups: list_groups::ListGroupProvider { pool: pool.clone() },
            sequences: sequences::SequenceProvider { pool: pool.clone() },
            maps: maps::MapsProvider { pool: pool.clone() },
            provenance: provenance::ProvenanceProvider { pool: pool.clone() },
            pool,
        })
    }
}
