pub mod collections;

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

    Ok(())
}
