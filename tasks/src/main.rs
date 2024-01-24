use clap::Parser;

pub mod admin;
pub mod dataset;
// pub mod search;
pub mod data;
pub mod reports;


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
    // #[command(subcommand)]
    // Search(search::Command),

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
        /// The global ID of the dataset to process the data as
        dataset: String,
        /// The path to the file being imported
        path: String,
        /// A list of dataset global IDs that the import can use when
        /// matching to existing data
        isolation_context: Vec<String>,
    },

    /// Perform tasks on raw data sets
    #[command(subcommand)]
    Data(data::Command),

    /// Create reports related to the database
    #[command(subcommand)]
    Reports(reports::Command),
}


fn main() {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match &cli.command {
        Commands::CreateAdmin { name, email, password } => admin::create_admin(name, email, password),
        // Commands::Search(command) => search::process_command(command),
        Commands::Data(command) => data::process_command(command),
        Commands::Reports(command) => reports::process_command(command),
        Commands::Dataset { worker, dataset, isolation_context, path } => dataset::import(worker, dataset, isolation_context, path),
    }
}
