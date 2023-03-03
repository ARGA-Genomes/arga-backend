use axum::response::{IntoResponse, Response};
use hyper::StatusCode;
use tracing::error;


#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("missing the '{0}' parameter in the request")]
    MissingParam(String),

    #[error("invalid configuration value for {0}. value = {1}")]
    Configuration(String, String),

    #[error("an internal server error occurred")]
    Internal(#[from] anyhow::Error),

    #[error("an error occurred with the solr search service")]
    Solr(#[from] crate::index::providers::solr::Error),
}


impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::MissingParam(_) => StatusCode::BAD_REQUEST,

            Error::Configuration(_, _) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Solr(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}


impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match &self {
            Error::Configuration(name, value) => {
                error!(name, value, "Invalid configuration value");
            },
            Error::Internal(err) => {
                error!(?err, "Internal error");
            },
            Error::Solr(err) => {
                error!(?err, "Solr error");
            },

            _ => {}
        }

        (self.status_code(), self.to_string()).into_response()
    }
}
