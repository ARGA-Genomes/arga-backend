pub mod extensions;

pub mod search;
pub mod class;
pub mod order;
pub mod family;
pub mod genus;
pub mod species;
pub mod stats;
pub mod maps;
pub mod lists;
pub mod sources;
pub mod datasets;
pub mod names;
pub mod assembly;
pub mod specimen;
pub mod markers;
pub mod overview;
pub mod taxa;
pub mod specimens;
pub mod subsamples;
pub mod dna_extracts;

pub use arga_core::{schema, schema_gnl, models, get_database_url};

use thiserror::Error;

use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;

use crate::http::Error as HttpError;

use self::extensions::pagination::Page;


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

    pub class: class::ClassProvider,
    pub order: order::OrderProvider,
    pub family: family::FamilyProvider,
    pub genus: genus::GenusProvider,
    pub markers: markers::MarkerProvider,
    pub overview: overview::OverviewProvider,
    pub stats: stats::StatsProvider,
    pub species: species::SpeciesProvider,
    pub assembly: assembly::AssemblyProvider,
    pub lists: lists::ListProvider,
    pub sources: sources::SourceProvider,
    pub datasets: datasets::DatasetProvider,
    pub taxa: taxa::TaxaProvider,
    pub specimens: specimens::SpecimenProvider,
    pub subsamples: subsamples::SubsampleProvider,
    pub dna_extracts: dna_extracts::DnaExtractProvider,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Database, Error> {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url);
        // let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_setup(url, establish_tls_connection);
        let pool = Pool::builder().build(config).await?;

        Ok(Database {
            class: class::ClassProvider { pool: pool.clone() },
            order: order::OrderProvider { pool: pool.clone() },
            family: family::FamilyProvider { pool: pool.clone() },
            genus: genus::GenusProvider { pool: pool.clone() },
            markers: markers::MarkerProvider { pool: pool.clone() },
            overview: overview::OverviewProvider { pool: pool.clone() },
            stats: stats::StatsProvider { pool: pool.clone() },
            species: species::SpeciesProvider { pool: pool.clone() },
            assembly: assembly::AssemblyProvider { pool: pool.clone() },
            lists: lists::ListProvider { pool: pool.clone() },
            sources: sources::SourceProvider { pool: pool.clone() },
            datasets: datasets::DatasetProvider { pool: pool.clone() },
            taxa: taxa::TaxaProvider { pool: pool.clone() },
            specimens: specimens::SpecimenProvider { pool: pool.clone() },
            subsamples: subsamples::SubsampleProvider { pool: pool.clone() },
            dna_extracts: dna_extracts::DnaExtractProvider { pool: pool.clone() },
            pool
        })
    }
}
