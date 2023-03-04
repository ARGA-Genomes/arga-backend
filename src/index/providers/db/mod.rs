pub mod search;
pub mod species;

use sqlx::PgPool;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum Error {
    #[error("internal database error")]
    Database(#[from] sqlx::Error),
}


#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}


impl Database {
    pub fn new(pool: PgPool) -> Database {
        Database { pool }
    }
}
