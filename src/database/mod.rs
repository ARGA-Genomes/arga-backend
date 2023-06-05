pub mod schema;
pub mod schema_gnl;

pub mod search;
pub mod family;
pub mod genus;
pub mod species;
pub mod stats;
pub mod maps;
pub mod lists;
pub mod names;
pub mod models;

use diesel::{ConnectionResult, Queryable};
use futures::FutureExt;
use futures::future::BoxFuture;
use thiserror::Error;

use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;

use crate::http::Error as HttpError;
use crate::index::Taxonomy;


#[derive(Error, Debug)]
pub enum Error {
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


#[derive(Queryable, Debug)]
struct Taxon {
    scientific_name_authorship: Option<String>,
    canonical_name: Option<String>,
    kingdom: Option<String>,
    phylum: Option<String>,
    class: Option<String>,
    order: Option<String>,
    family: Option<String>,
    genus: Option<String>,
}

impl From<Taxon> for Taxonomy {
    fn from(source: Taxon) -> Self {
        Self {
            scientific_name: "".to_string(),
            canonical_name: source.canonical_name,
            authorship: source.scientific_name_authorship,

            kingdom: source.kingdom,
            phylum: source.phylum,
            class: source.class,
            order: source.order,
            family: source.family,
            genus: source.genus,
            vernacular_group: None,
        }
    }
}


#[derive(Clone)]
pub struct Database {
    pub pool: Pool<AsyncPgConnection>,
}


impl Database {
    pub async fn connect(url: &str) -> Result<Database, Error> {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url);
        // let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_setup(url, establish_tls_connection);
        let pool = Pool::builder().build(config).await?;

        Ok(Database { pool })
    }
}


fn _establish_tls_connection(url: &str) -> BoxFuture<ConnectionResult<AsyncPgConnection>> {
    (async {
        let store = rustls::RootCertStore::empty();
        let config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(store)
            .with_no_client_auth();

        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(config);
        let (client, connection) = tokio_postgres::connect(url, tls).await.map_err(|e| {
            diesel::ConnectionError::BadConnection(e.to_string())
        })?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        AsyncPgConnection::try_from(client).await
    }).boxed()
}