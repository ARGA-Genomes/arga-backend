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
pub mod assembly;
pub mod specimen;
pub mod markers;
pub mod models;

use std::marker::PhantomData;

use diesel::expression::{ValidGrouping, AsExpression};
use diesel::pg::Pg;
use diesel::query_builder::{QueryFragment, AstPass, QueryId};
use diesel::sql_types::{BigInt, SqlType, SingleValue};
use diesel::{ConnectionResult, Queryable, QueryResult, Expression, DieselNumericOps, SelectableExpression, AppearsOnTable};
use futures::FutureExt;
use futures::future::BoxFuture;
use thiserror::Error;

use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;

use crate::http::Error as HttpError;
use crate::index::Taxonomy;


pub type PgPool = Pool<AsyncPgConnection>;


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


pub fn sum_if<T, E>(expr: E) -> ColumnSum<T, E::Expression>
where
    T: SqlType + SingleValue,
    E: AsExpression<T>,
{
    ColumnSum {
        expr: expr.as_expression(),
        _marker: PhantomData,
    }
}


#[derive(Debug, Clone, Copy, QueryId, DieselNumericOps)]
pub struct ColumnSum<T, E> {
    expr: E,
    _marker: PhantomData<T>,
}

impl<T, E> QueryFragment<Pg> for ColumnSum<T, E>
where
    T: SqlType + SingleValue,
    E: QueryFragment<Pg>,
{
    fn walk_ast<'b>(&'b self, mut out: AstPass<'_, 'b, Pg>) -> QueryResult<()> {
        out.push_sql("SUM(CASE WHEN ");
        self.expr.walk_ast(out.reborrow())?;
        out.push_sql(" THEN 1 ELSE 0 END)");
        Ok(())
    }
}



impl<T, E> Expression for ColumnSum<T, E>
where
    T: SqlType + SingleValue,
    E: Expression,
{
    type SqlType = BigInt;
}

impl<T, E, GB> ValidGrouping<GB> for ColumnSum<T, E>
where T: SqlType + SingleValue,
{
    type IsAggregate = diesel::expression::is_aggregate::Yes;
}

impl<T, E, QS> SelectableExpression<QS> for ColumnSum<T, E>
where
    Self: AppearsOnTable<QS>,
    E: SelectableExpression<QS>,
{
}

impl<T, E, QS> AppearsOnTable<QS> for ColumnSum<T, E>
where
    Self: Expression,
    E: AppearsOnTable<QS>,
{
}



#[derive(Clone)]
pub struct Database {
    pub pool: Pool<AsyncPgConnection>,

    pub genus: genus::GenusProvider,
    pub markers: markers::MarkerProvider,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Database, Error> {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url);
        // let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_setup(url, establish_tls_connection);
        let pool = Pool::builder().build(config).await?;

        Ok(Database {
            genus: genus::GenusProvider { pool: pool.clone() },
            markers: markers::MarkerProvider { pool: pool.clone() },
            pool
        })
    }
}


pub fn get_database_url() -> String {
    match std::env::var("DATABASE_URL") {
        Ok(url) => url.to_string(),
        Err(_) => {
            tracing::info!("DATABASE_URL not specified. Building URL from other env vars");
            let host = std::env::var("DATABASE_HOST").expect("Must specify a database host");
            let port = std::env::var("DATABASE_PORT").expect("Must specify a database port");
            let user = std::env::var("DATABASE_USER").expect("Must specify a database user");
            let pass = std::env::var("DATABASE_PASS").expect("Must specify a database pass");
            let name = std::env::var("DATABASE_NAME").expect("Must specify a database name");

            format!("postgresql://{user}:{pass}@{host}:{port}/{name}")
        }
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
