pub mod search;
pub mod species;
pub mod stats;

use diesel::ConnectionResult;
use futures::FutureExt;
use futures::future::BoxFuture;
use thiserror::Error;

use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;


#[derive(Error, Debug)]
pub enum Error {
    #[error("internal database error")]
    Connection(#[from] diesel::result::Error),
    #[error("internal database pool error")]
    Pool(#[from] diesel_async::pooled_connection::PoolError),
    #[error("internal database pool error")]
    GetPool(#[from] diesel_async::pooled_connection::bb8::RunError),
}


#[derive(Clone)]
pub struct Database {
    pool: Pool<AsyncPgConnection>,
}


impl Database {
    pub async fn connect(url: &str) -> Result<Database, Error> {
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_setup(url, establish_tls_connection);
        let pool = Pool::builder().build(config).await?;

        Ok(Database { pool })
    }
}


fn establish_tls_connection(url: &str) -> BoxFuture<ConnectionResult<AsyncPgConnection>> {
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
