pub mod taxa;

use std::path::PathBuf;


#[derive(clap::Subcommand)]
pub enum Command {
    /// Create a report showing taxon match success/failure
    MatchTaxa {
        /// The taxa CSV file
        input: String,
    }
}

pub fn process_command(command: &Command) {
    match command {
        Command::MatchTaxa { input } => taxa::match_report(PathBuf::from(input)).unwrap(),
    }
}


#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Csv(csv::Error),
    Database(diesel::result::Error),
    Pool(diesel::r2d2::PoolError),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<csv::Error> for Error {
    fn from(value: csv::Error) -> Self {
        Self::Csv(value)
    }
}

impl From<diesel::result::Error> for Error {
    fn from(value: diesel::result::Error) -> Self {
        Self::Database(value)
    }
}

impl From<diesel::r2d2::PoolError> for Error {
    fn from(value: diesel::r2d2::PoolError) -> Self {
        Self::Pool(value)
    }
}
