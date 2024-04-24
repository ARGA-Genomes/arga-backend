use std::path::PathBuf;

pub mod formatting;
pub mod treatments;


#[derive(clap::Subcommand)]
pub enum Command {
    ImportTreatments { input_dir: String },
}

pub fn process_command(command: &Command) {
    match command {
        Command::ImportTreatments { input_dir } => treatments::import(PathBuf::from(input_dir)).unwrap(),
    }
}
