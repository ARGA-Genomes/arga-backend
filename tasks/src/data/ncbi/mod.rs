use std::path::PathBuf;

pub mod name_matcher;
pub mod biosamples;


#[derive(clap::Subcommand)]
pub enum Command {
    /// Convert a biosamples XML into event CSV files
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
        Command::ConvertBiosamples { input } => biosamples::convert(PathBuf::from(input)).unwrap(),
        Command::SummariseBiosamples { input } => biosamples::summarise(PathBuf::from(input)).unwrap(),
    }
}
