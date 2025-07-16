// mod attributes;
mod common;
// mod csv_upload;
// mod datasets;
mod media;
// mod sources;
mod taxa;

use axum::routing::{get, post};
use axum::{Json, Router};
use axum_login::AuthManagerLayerBuilder;
use axum_login::tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use tower_sessions::cookie::Key;
use tracing::instrument;

use crate::database::models;
use crate::http::auth::{AuthSession, Credentials, DatabaseUserStore};
use crate::http::{Context, Error};


/// The REST gateway for the admin backend for basic CRUD operations
pub(crate) fn router(context: Context) -> Router<Context> {
    // for cookie signing
    let key = Key::generate();

    // auth session management. we use a memory store because it's an admin backend and
    // requiring a login every production deploy is easier to manage
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(time::Duration::days(7)))
        .with_signed(key);
    // .with_same_site(tower_sessions::cookie::SameSite::Lax);

    let auth_backend = DatabaseUserStore::new(context.database.pool);
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend, session_layer).build();

    // unlike tower services the layers in the axum router is applied outside in like an onion.
    // so the layer and routes at the bottom will be applied before the ones at the top, which means
    // that protected routes have to be added ABOVE the login_required! route layer
    Router::new()
        .merge(media::router())
        .merge(taxa::router())
        .route_layer(axum_login::login_required!(DatabaseUserStore))
        .route("/logout", get(logout_handler))
        .route("/login", post(login_handler))
        .layer(auth_layer)
}


#[instrument(skip_all)]
async fn login_handler(
    mut auth_session: AuthSession,
    Json(creds): Json<Credentials>,
) -> Result<Json<models::User>, Error> {
    tracing::debug!(creds.email, "Attempting login");

    let user = match auth_session.authenticate(creds).await? {
        Some(user) => user,
        None => {
            tracing::error!("Login failed. Invalid credentials");
            Err(Error::Authentication)?
        }
    };

    auth_session.login(&user).await?;
    Ok(Json(user.into()))
}

async fn logout_handler(mut auth_session: AuthSession) -> Result<(), Error> {
    tracing::debug!(user=?auth_session.user, "Logging out user");
    auth_session.logout().await?;
    Ok(())
}
