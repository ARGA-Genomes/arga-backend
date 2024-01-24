pub mod traces;

use std::path::PathBuf;


#[derive(clap::Subcommand)]
pub enum Command {
    /// Download traces files from a CSV file
    DownloadTraces {
        /// The BOLD csv file with trace links
        input: String,
        /// The output directory to store all trace files
        output: String,
    },

    /// Process trace files
    ProcessTraces {
        /// The BOLD csv file with trace links
        input: String,
        /// The path to the directory containing the downloaded trace files
        traces: String,
    }
}

pub fn process_command(command: &Command) {
    match command {
        Command::DownloadTraces { input, output } => traces::download(
            PathBuf::from(input),
            PathBuf::from(output),
        ).unwrap(),
        Command::ProcessTraces { input, traces } => traces::process(
            PathBuf::from(input),
            PathBuf::from(traces),
        ).unwrap(),
    }
}
