use arga_core::schema;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::{RunQueryDsl, *};


/// Create a new admin user
///
/// Admin users can access the admin curation frontend but other than that
/// its not used within the rest of the backend since the vast majority is
/// open access
pub fn create_admin(name: &str, email: &str, password: &str) {
    use schema::users;

    let url = arga_core::get_database_url();
    let manager = ConnectionManager::<PgConnection>::new(url);
    let pool = Pool::builder().build(manager).expect("Could not build connection pool");
    let mut conn = pool.get().expect("Could not checkout connection");

    let hash = password_auth::generate_hash(password.as_bytes());

    diesel::insert_into(users::table)
        .values((
            users::name.eq(name),
            users::email.eq(email),
            users::password_hash.eq(hash),
            users::user_role.eq("admin"),
        ))
        .execute(&mut conn)
        .expect("Failed to insert admin user");
}
