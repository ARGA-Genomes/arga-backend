use std::path::PathBuf;

pub mod name_matcher;
pub mod biosamples;


#[derive(clap::Subcommand)]
pub enum Command {
    /// Import a biosamples XML
    ImportBiosamples {
        /// The biosamples XML file
        input: String,
    },

    /// Convert a biosamples XML into a event CSV files
    ConvertBiosamples {
        /// The biosamples XML file
        input: String,
    },

    /// Summarise a biosamples XML file
    SummariseBiosamples {
        /// The biosamples XML file
        input: String,
    }
}

pub fn process_command(command: &Command) {
    match command {
        Command::ImportBiosamples { input } => biosamples::import(PathBuf::from(input)).unwrap(),
        Command::ConvertBiosamples { input } => biosamples::convert(PathBuf::from(input)).unwrap(),
        Command::SummariseBiosamples { input } => biosamples::summarise(PathBuf::from(input)).unwrap(),
    }
}
