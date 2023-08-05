pub mod admin;
pub mod dataset;
pub mod search;


/// The ARGA backend
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
   #[command(subcommand)]
   command: Option<Commands>,
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


#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv().ok();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::CreateAdmin { name, email, password }) => tasks::admin::create_admin(name, email, password).await,
        Some(Commands::Search(command)) => search::process_command(command).await,
        Some(Commands::Dataset { worker, name, path }) => tasks::dataset::import(worker, name, path).await?,
        None => serve().await,
    }

    Ok(())
}
