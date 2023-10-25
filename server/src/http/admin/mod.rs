mod common;
mod taxa;
mod csv_upload;
mod media;
mod lists;

use argon2::{PasswordHash, Argon2, PasswordVerifier};
use axum::extract::State;
use axum::{Router, Json};
use axum::routing::{get, post};
use axum_login::axum_sessions::SessionLayer;
use axum_login::axum_sessions::async_session::MemoryStore;

use axum_login::secrecy::ExposeSecret;
use serde::Deserialize;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use tracing::instrument;

use crate::http::{Context, error};
use crate::database::{schema, models, Database};

use super::auth::{AuthContext, AuthLayer, DatabaseUserStore, User, RequireAuth};


/// The REST gateway for the admin backend for basic CRUD operations
pub(crate) fn router(context: Context) -> Router<Context> {
    let secret = [0; 64];

    let session_store = MemoryStore::new();
    let session_layer = SessionLayer::new(session_store, &secret[..]);

    let user_store = DatabaseUserStore::new(context.database.pool.clone());
    let auth_layer = AuthLayer::new(user_store, &secret[..]);


    Router::new()
        .route("/api/admin/logout", get(logout_handler))
        .merge(taxa::router())
        .merge(csv_upload::router())
        .merge(media::router())
        .merge(lists::router())
        .route_layer(RequireAuth::login())
        .route("/api/admin/login", post(login_handler))
        .layer(auth_layer)
        .layer(session_layer)
}


#[derive(Deserialize)]
struct LoginForm {
    email: String,
    password: String,
}

#[instrument(skip_all)]
async fn login_handler(
    mut auth: AuthContext,
    State(db_provider): State<Database>,
    Json(form): Json<LoginForm>,
) -> Result<Json<models::User>, super::error::Error>
{
    use schema::users::dsl::*;
    tracing::debug!(form.email, "Attempting login");

    let mut conn = db_provider.pool.get().await.unwrap();
    let record: User = users.filter(email.eq(form.email)).get_result(&mut conn).await.unwrap();
    tracing::debug!(?record);

    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(&record.password_hash.expose_secret()).unwrap();

    match argon2.verify_password(form.password.as_bytes(), &parsed_hash) {
        Ok(_) => {
            auth.login(&record).await.unwrap();

            Ok(Json(models::User {
                id: uuid::Uuid::parse_str(&record.id).unwrap(),
                name: record.name,
                email: record.email,
            }))
        },
        Err(err) => {
            tracing::error!(?err, "Login failed");
            Err(error::Error::Authentication)
        },
    }
}

async fn logout_handler(mut auth: AuthContext) {
    tracing::debug!(user=?auth.current_user, "Logging out user");
    auth.logout().await;
}
