use async_trait::async_trait;
use axum_login::{AuthUser, AuthnBackend, UserId};
use diesel::prelude::*;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use uuid::Uuid;

use crate::database::{models, schema};


pub type AuthSession = axum_login::AuthSession<DatabaseUserStore>;


#[derive(thiserror::Error, Debug)]
enum AuthError {
    #[error("the user role '{0}' is not valid")]
    InvalidRole(String),
}


#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Role {
    User,
    Admin,
}


#[derive(Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("email", &self.email)
            .field("password", &"[redacted]")
            .finish()
    }
}


#[derive(Debug, Clone)]
pub struct FullUser {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub user_role: Role,
    // a secret string has some guarantees around cloning and printing
    // so that sensitive fields are harder to misuse. for more serious
    // protections switch to https://github.com/stouset/secrets which includes
    // kernel level memory protections that adds more defence in depth
    pub password_hash: SecretString,
    pub session_id: Option<String>,
}

// we manually implement the querable here to immediately store password
// hashes in a secrect store and assert the user role
impl Queryable<schema::users::SqlType, diesel::pg::Pg> for FullUser {
    type Row = (Uuid, String, String, String, String, Option<String>);

    fn build(row: Self::Row) -> diesel::deserialize::Result<Self> {
        let role = match row.3.as_ref() {
            "admin" => Ok(Role::Admin),
            "user" => Ok(Role::User),
            other => Err(AuthError::InvalidRole(other.to_string())),
        }?;

        Ok(FullUser {
            id: row.0,
            name: row.1,
            email: row.2,
            user_role: role,
            password_hash: row.4.into(),
            session_id: row.5,
        })
    }
}

impl From<FullUser> for models::User {
    fn from(value: FullUser) -> Self {
        Self {
            id: value.id,
            name: value.name,
            email: value.email,
        }
    }
}


impl AuthUser for FullUser {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id.clone()
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash.expose_secret().as_bytes()
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


// TODO: remove the async_trait dependency after the next release of axum_login
#[async_trait]
impl AuthnBackend for DatabaseUserStore {
    type Credentials = Credentials;
    type Error = super::Error;
    type User = FullUser;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        use schema::users::dsl::*;
        tracing::debug!(creds.email, "Loading user");

        let mut conn = self.pool.get().await?;

        let user: Self::User = users.filter(email.eq(creds.email)).get_result(&mut conn).await?;

        // verifying passwords are necessarily constant time so we spin it out to prevent it blocking
        let user = tokio::task::spawn_blocking(|| {
            match password_auth::verify_password(creds.password, &user.password_hash.expose_secret()) {
                Ok(_) => Some(user),
                Err(_) => None,
            }
        })
        .await
        .map_err(|err| super::Error::Internal(err.into()))?;

        Ok(user)
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        use schema::users::dsl::*;
        let mut conn = self.pool.get().await?;

        let user: Option<Self::User> = users.filter(id.eq(user_id)).get_result(&mut conn).await.optional()?;
        Ok(user)
    }
}


impl From<password_auth::VerifyError> for super::Error {
    fn from(_value: password_auth::VerifyError) -> Self {
        Self::Authentication
    }
}


impl From<axum_login::Error<DatabaseUserStore>> for super::Error {
    fn from(_value: axum_login::Error<DatabaseUserStore>) -> Self {
        Self::Authentication
    }
}
