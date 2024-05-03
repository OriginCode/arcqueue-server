use actix_web::{http::StatusCode, HttpResponse, ResponseError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("an unspecified internal error occurred: {0}")]
    Internal(#[from] anyhow::Error),
    #[error("an database error occurred or resource not found")]
    Database(#[from] sqlx::Error),
    #[error("failed to parse uuid: {0}")]
    UuidParseFailure(#[from] sqlx::types::uuid::Error),
    #[error("n needed to be greater than 1")]
    NLessThanOne,
    #[error("player name already in queue")]
    NameAlreadyInQueue,
    #[error("player name not in queue")]
    NameNotInQueue,
    #[error("player is already the last one in the queue")]
    NameAlreadyLast,
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match &self {
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Database(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UuidParseFailure(_) => StatusCode::BAD_REQUEST,
            Self::NLessThanOne => StatusCode::BAD_REQUEST,
            Self::NameAlreadyInQueue => StatusCode::BAD_REQUEST,
            Self::NameNotInQueue => StatusCode::BAD_REQUEST,
            Self::NameAlreadyLast => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}
