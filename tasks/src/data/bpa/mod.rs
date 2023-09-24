pub mod collections;
pub mod accessions;
pub mod subsamples;
pub mod extractions;
pub mod sequences;
pub mod assemblies;
pub mod annotations;
pub mod depositions;

use std::path::PathBuf;

use super::Error;


#[derive(clap::Subcommand)]
pub enum Command {
    /// Normalise like fields in a BPA csv file
    Normalise {
        /// The BPA csv file
        input: String,
    },
}

pub fn process_command(command: &Command) {
    match command {
        Command::Normalise { input } => normalise(PathBuf::from(input)).unwrap(),
    }
}

fn normalise(path: PathBuf) -> Result<(), Error> {
    collections::normalise(&path)?;
    accessions::normalise(&path)?;
    subsamples::normalise(&path)?;
    extractions::normalise(&path)?;
    sequences::normalise(&path)?;
    assemblies::normalise(&path)?;
    annotations::normalise(&path)?;
    depositions::normalise(&path)?;

    Ok(())
}
