#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("an error occurred with the database connection")]
    Database(#[from] diesel::result::Error),

    #[error("an error occurred parsing the file")]
    Csv(#[from] csv::Error),

    #[error("an error occurred getting a database connection")]
    Pool(#[from] diesel::r2d2::PoolError),

    #[error(transparent)]
    Parsing(#[from] ParseError),
}


#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("failed to parse coordinates: {0}")]
    Coordinates(String),

    #[error("invalid value: {0}")]
    InvalidValue(String),

    #[error(transparent)]
    DateTime(#[from] chrono::ParseError),
}
