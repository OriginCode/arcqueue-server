use actix_web::{http::StatusCode, HttpResponse, ResponseError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("an unspecified internal error occurred: {0}")]
    InternalError(#[from] anyhow::Error),
    #[error("an database error occurred or resource not found")]
    DatabaseError(#[from] sqlx::Error),
    #[error("failed to parse uuid: {0}")]
    UuidParseError(#[from] sqlx::types::uuid::Error),
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match &self {
            Self::InternalError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::DatabaseError(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND,
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UuidParseError(_) => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).body(self.to_string())
    }
}
