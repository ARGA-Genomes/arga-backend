pub mod bpa;
pub mod ncbi;
// pub mod bold;
// pub mod oplogger;
// pub mod plazi;

use tracing::{instrument, info};

#[derive(clap::Subcommand)]
pub enum Command {
    /// Extra processing for NCBI datasets
    #[command(subcommand)]
    Ncbi(ncbi::Command),
    /// Extra processing for BPA datasets
    #[command(subcommand)]
    Bpa(bpa::Command),
    // Extra processing for BOLD datasets
    // #[command(subcommand)]
    // Bold(bold::Command),
    // #[command(subcommand)]
    // Plazi(plazi::Command),
    // #[command(subcommand)]
    // Oplog(oplogger::Command),
}

#[instrument(skip_all)]
pub fn process_command(command: &Command) {
    tracing_subscriber::fmt().init();
    info!("Processing data command");

    match command {
        Command::Ncbi(cmd) => {
            info!("Processing NCBI command");
            ncbi::process_command(cmd)
        },
        Command::Bpa(cmd) => {
            info!("Processing BPA command");
            bpa::process_command(cmd)
        },
        // Command::Bold(cmd) => bold::process_command(cmd),
        // Command::Plazi(cmd) => plazi::process_command(cmd),
        // Command::Oplog(cmd) => oplogger::process_command(cmd),
    }
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Parser(quick_xml::Error),
    GeoParser(String),
    Csv(csv::Error),
    Database(diesel::result::Error),
    Pool(diesel::r2d2::PoolError),
    Parsing(ParseError),
    // Http(ureq::Error),
    // Abif(abif::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<quick_xml::Error> for Error {
    fn from(value: quick_xml::Error) -> Self {
        Self::Parser(value)
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

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("invalid value: {0}")]
    InvalidValue(String),

    #[error(transparent)]
    DateTime(#[from] chrono::ParseError),

    #[error("value not found: {0}")]
    NotFound(String),

    #[error("invalid structure: {0}")]
    InvalidStructure(String),
}

// impl From<ureq::Error> for Error {
//     fn from(value: ureq::Error) -> Self {
//         Self::Http(value)
//     }
// }

// impl From<abif::Error> for Error {
//     fn from(value: abif::Error) -> Self {
//         Self::Abif(value)
//     }
// }
