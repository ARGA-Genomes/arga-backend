pub mod crdt;
pub mod models;
pub mod schema;
pub mod schema_gnl;
pub mod search;

pub fn get_database_url() -> String {
    match std::env::var("DATABASE_URL") {
        Ok(url) => url.to_string(),
        Err(_) => {
            let host = std::env::var("DATABASE_HOST").expect("Must specify a database host");
            let port = std::env::var("DATABASE_PORT").expect("Must specify a database port");
            let user = std::env::var("DATABASE_USER").expect("Must specify a database user");
            let pass = std::env::var("DATABASE_PASS").expect("Must specify a database pass");
            let name = std::env::var("DATABASE_NAME").expect("Must specify a database name");

            format!("postgresql://{user}:{pass}@{host}:{port}/{name}")
        }
    }
}
