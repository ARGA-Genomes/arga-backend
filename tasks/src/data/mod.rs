pub mod ncbi;


#[derive(clap::Subcommand)]
pub enum Command {
    /// Extra processing for NCBI datasets
    #[command(subcommand)]
    Ncbi(ncbi::Command),
}

pub fn process_command(command: &Command) {
    tracing_subscriber::fmt().init();

    match command {
        Command::Ncbi(cmd) => ncbi::process_command(cmd),
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
