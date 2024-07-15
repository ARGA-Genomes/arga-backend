use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use tracing::error;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("missing the '{0}' parameter in the request")]
    MissingParam(String),

    #[error("the resource '{0}' could not found")]
    NotFound(String),

    #[error("invalid configuration value for {0}. value = {1}")]
    Configuration(String, String),

    #[error("invalid data found for {0} in the record {1}. id = {2}")]
    InvalidData(String, String, String),

    #[error("an authentication error occurred")]
    Authentication,

    #[error(transparent)]
    Internal(#[from] anyhow::Error),

    #[error(transparent)]
    Database(crate::database::Error),

    #[error(transparent)]
    SearchIndex(#[from] arga_core::search::Error),

    #[error("request timeout")]
    Timeout,

    #[error("upstream request timeout")]
    GatewayTimeout,

    #[error(transparent)]
    GraphQL(#[from] async_graphql::DeserializerError),

    #[error(transparent)]
    GeoJSON(#[from] geojson::Error),
}

impl From<crate::database::Error> for Error {
    fn from(err: crate::database::Error) -> Self {
        // we want to treate not found errors a little differently so that we can
        // log the resource attempting to load
        match err {
            crate::database::Error::NotFound(resource) => Error::NotFound(resource),
            err => Error::Database(err),
        }
    }
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::MissingParam(_) => StatusCode::BAD_REQUEST,
            Error::GraphQL(_) => StatusCode::BAD_REQUEST,
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::InvalidData(_, _, _) => StatusCode::INTERNAL_SERVER_ERROR,

            Error::Configuration(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Authentication => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::SearchIndex(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::GeoJSON(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Timeout => StatusCode::REQUEST_TIMEOUT,
            Error::GatewayTimeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match &self {
            Error::Configuration(name, value) => {
                error!(name, value, "Invalid configuration value");
            }
            Error::Authentication => {
                error!("Authentication error");
            }
            Error::Internal(err) => {
                error!(?err, "Internal error");
            }
            Error::Database(err) => {
                error!(?err, "Database error");
            }
            Error::SearchIndex(err) => {
                error!(?err, "Search index error");
            }
            Error::Timeout => {
                tracing::debug!("Timeout");
                error!("Timeout");
            }
            Error::GatewayTimeout => {
                tracing::debug!("Gateway timeout");
                error!("Gateway timeout");
            }
            Error::GraphQL(err) => {
                error!(?err, "Deserializer error");
            }
            Error::GeoJSON(err) => {
                error!(?err, "GeoJSON error");
            }

            _ => {}
        }

        (self.status_code(), self.to_string()).into_response()
    }
}

pub struct InternalError(anyhow::Error);

impl IntoResponse for InternalError {
    fn into_response(self) -> Response {
        Error::Internal(self.0).into_response()
    }
}

impl<E> From<E> for InternalError
where
    E: Into<anyhow::Error>,
{
    fn from(source: E) -> Self {
        Self(source.into())
    }
}
