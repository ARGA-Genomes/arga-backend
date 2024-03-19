pub mod ncbi;
pub mod bpa;
// pub mod bold;
pub mod oplogger;


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

    #[command(subcommand)]
    Oplog(oplogger::Command),
}

pub fn process_command(command: &Command) {
    tracing_subscriber::fmt().init();

    match command {
        Command::Ncbi(cmd) => ncbi::process_command(cmd),
        Command::Bpa(cmd) => bpa::process_command(cmd),
        // Command::Bold(cmd) => bold::process_command(cmd),
        Command::Oplog(cmd) => oplogger::process_command(cmd),
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
