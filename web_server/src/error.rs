use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use actix_web::error::QueryPayloadError;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(unused)]
pub enum CustomError {
    #[error("There was an error parsing the input: {0}")]
    ParseInputDataError(QueryPayloadError),
    #[error("An internal error occurred. Please try again later.")]
    InternalError,
    #[error("An internal database error occurred: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("An internal server error occurred: {0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("An internal server error occurred: {0}")]
    StdIoError(#[from] std::io::Error),
    #[error("An internal server error occurred: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Invalid credentials.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error("{0}")]
    Duplicate(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("{0}")]
    NotFound(String)
}


impl error::ResponseError for CustomError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            CustomError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::ParseInputDataError(_) => StatusCode::BAD_REQUEST,
            CustomError::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::StdIoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::ReqwestError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            CustomError::Duplicate(_) => StatusCode::CONFLICT,
            CustomError::Forbidden(_) => StatusCode::FORBIDDEN,
            CustomError::NotFound(_) => StatusCode::NOT_FOUND,
            CustomError::InvalidCredentials(_) => StatusCode::UNAUTHORIZED
        }
    }
}