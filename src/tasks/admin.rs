use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;

use axum_login::secrecy::{SecretString, ExposeSecret};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use arga_backend::database::{schema, Database};


/// Create a new admin user
///
/// Admin users can access the admin curation frontend but other than that
/// its not used within the rest of the backend since the vast majority is
/// open access
pub async fn create_admin(name: &str, email: &str, password: &str) {
    let password = SecretString::new(password.to_string());

    use schema::users::dsl as dsl;

    let db_host = arga_backend::database::get_database_url();
    let database = Database::connect(&db_host).await.expect("Failed to connect to the database");
    let mut pool = database.pool.get().await.unwrap();

    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2.hash_password(&password.expose_secret().as_bytes(), &salt).unwrap().to_string();

    diesel::insert_into(dsl::users)
        .values((
            dsl::name.eq(name),
            dsl::email.eq(email),
            dsl::password_hash.eq(hash),
            dsl::password_salt.eq(salt.to_string()),
            dsl::user_role.eq("admin"),
        ))
        .execute(&mut pool)
        .await.unwrap();
}
