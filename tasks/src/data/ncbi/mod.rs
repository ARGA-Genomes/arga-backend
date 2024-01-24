use std::path::PathBuf;

pub mod name_matcher;
pub mod biosamples;


#[derive(clap::Subcommand)]
pub enum Command {
    /// Convert a biosamples XML into event CSV files
    ConvertBiosamples {
        /// The biosamples XML file
        input: String,

        /// The output directory for biosample CSV event files
        out: String,
    },

    /// Summarise a biosamples XML file
    SummariseBiosamples {
        /// The biosamples XML file
        input: String,
    }
}

pub fn process_command(command: &Command) {
    match command {
        Command::ConvertBiosamples { input, out } => biosamples::convert(PathBuf::from(input), PathBuf::from(out)).unwrap(),
        Command::SummariseBiosamples { input } => biosamples::summarise(PathBuf::from(input)).unwrap(),
    }
}
