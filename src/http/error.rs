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

    #[error("an authentication error occurred")]
    Authentication,

    #[error("an internal server error occurred")]
    Internal(#[from] anyhow::Error),

    #[error("an error occurred with the solr search service")]
    Solr(#[from] crate::index::providers::solr::Error),

    #[error("an error occurred with the database service")]
    Database(#[from] crate::index::providers::db::Error),

    #[error("an error occurred with the search index service")]
    SearchIndex(#[from] crate::index::providers::search::Error),
}


impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::MissingParam(_) => StatusCode::BAD_REQUEST,
            Error::NotFound(_) => StatusCode::NOT_FOUND,

            Error::Configuration(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Authentication => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Solr(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::SearchIndex(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}


impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match &self {
            Error::Configuration(name, value) => {
                error!(name, value, "Invalid configuration value");
            },
            Error::Authentication => {
                error!("Authentication error");
            },
            Error::Internal(err) => {
                error!(?err, "Internal error");
            },
            Error::Solr(err) => {
                error!(?err, "Solr error");
            },
            Error::Database(err) => {
                error!(?err, "Database error");
            },
            Error::SearchIndex(err) => {
                error!(?err, "Search index error");
            },

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
where E: Into<anyhow::Error>
{
    fn from(source: E) -> Self {
        Self(source.into())
    }
}
