use async_trait::async_trait;
use axum_login::{AuthUser, UserStore};
use axum_login::secrecy::{SecretString, SecretVec, ExposeSecret};

use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::bb8::Pool;
use tracing::instrument;
use uuid::Uuid;

use crate::schema;


pub type AuthContext = axum_login::extractors::AuthContext<User, DatabaseUserStore, Role>;
pub type AuthLayer = axum_login::AuthLayer<DatabaseUserStore, User, Role>;
pub type RequireAuth = axum_login::RequireAuthorizationLayer<User, Role>;


#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("the user role '{0}' is not valid")]
    InvalidRole(String)
}


#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Role {
    User,
    Admin,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub user_role: Role,
    pub password_hash: SecretString,
    pub password_salt: SecretString,
}

impl Queryable<schema::users::SqlType, diesel::pg::Pg> for User {
    type Row = (Uuid, String, String, String, String, String);

    fn build(row: Self::Row) -> diesel::deserialize::Result<Self> {
        let role = match row.3.as_ref() {
            "admin" => Ok(Role::Admin),
            "user" => Ok(Role::User),
            other => Err(Error::InvalidRole(other.to_string())),
        }?;

        Ok(User {
            id: row.0.to_string(),
            name: row.1,
            email: row.2,
            user_role: role,
            password_hash: SecretString::new(row.4),
            password_salt: SecretString::new(row.5),
        })
    }
}



impl<Role> AuthUser<Role> for User
where
    Role: PartialOrd + PartialEq + Clone + Send + Sync + 'static,
{
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_password_hash(&self) -> SecretVec<u8> {
        SecretVec::new(self.password_hash.expose_secret().clone().into())
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseUserStore {
    pool: Pool<AsyncPgConnection>,
}

impl DatabaseUserStore {
    pub fn new(pool: Pool<AsyncPgConnection>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<Role> UserStore<Role> for DatabaseUserStore
where
    Role: PartialOrd + PartialEq + Clone + Send + Sync + 'static,
{
    type User = User;

    #[instrument(skip(self))]
    async fn load_user(&self, user_id: &str) -> Result<Option<Self::User>, eyre::Error> {
        use schema::users::dsl::*;
        tracing::debug!(user_id, "Loading user");

        let uuid = Uuid::parse_str(user_id)?;
        let mut conn = self.pool.get().await?;
        let record = users.filter(id.eq(uuid)).get_result(&mut conn).await?;

        Ok(Some(record))
    }
}
