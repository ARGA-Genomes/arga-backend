pub mod search;
pub mod species;

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
        let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url);
        let pool = Pool::builder().build(config).await?;

        Ok(Database { pool })
    }
}
