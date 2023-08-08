use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;

use diesel::*;
use diesel::RunQueryDsl;
use diesel::r2d2::{ConnectionManager, Pool};

use arga_core::schema;


/// Create a new admin user
///
/// Admin users can access the admin curation frontend but other than that
/// its not used within the rest of the backend since the vast majority is
/// open access
pub fn create_admin(name: &str, email: &str, password: &str) {
    use schema::users::dsl as dsl;

    let url = arga_core::get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    let pool = Pool::builder().build(manager).expect("Could not build connection pool");
    let mut conn = pool.get().expect("Could not checkout connection");

    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2.hash_password(&password.as_bytes(), &salt).unwrap().to_string();

    diesel::insert_into(dsl::users)
        .values((
            dsl::name.eq(name),
            dsl::email.eq(email),
            dsl::password_hash.eq(hash),
            dsl::password_salt.eq(salt.to_string()),
            dsl::user_role.eq("admin"),
        ))
        .execute(&mut conn)
        .expect("Failed to insert admin user");
}
