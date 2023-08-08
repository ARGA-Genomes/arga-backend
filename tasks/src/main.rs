use clap::Parser;

pub mod admin;
pub mod dataset;
pub mod search;


/// The ARGA backend
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
   #[command(subcommand)]
   command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Run and manage the search index
    #[command(subcommand)]
    Search(search::Command),

    /// Create a new admin user
    CreateAdmin {
        /// The full name of the new admin user
        name: String,
        /// The email address of the new admin user
        email: String,
        /// A generated and safe password for the new admin user
        password: String,
    },

    /// Queue an import job for a dataset
    Dataset {
        /// The worker job type that should process the file
        worker: String,
        /// The name to give this dataset
        name: String,
        /// The path to the file being imported
        path: String,
    }
}


fn main() {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match &cli.command {
        Commands::CreateAdmin { name, email, password } => admin::create_admin(name, email, password),
        Commands::Search(command) => search::process_command(command),
        Commands::Dataset { worker, name, path } => dataset::import(worker, name, path),
    }
}
