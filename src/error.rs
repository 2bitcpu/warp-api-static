use std::convert::Infallible;
use warp::{body::BodyDeserializeError, http::StatusCode, Rejection};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum AppError {
    NotFound,
    Unauthorized,
    InvalidToken,
    BadRequest,
    Duplicate,
    InternalServerError,
    ServiceUnavailable,
}

impl warp::reject::Reject for AppError {}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AppError::NotFound => write!(f, "Not found"),
            AppError::Unauthorized => write!(f, "Unauthorized"),
            AppError::InvalidToken => write!(f, "Invalid token"),
            AppError::BadRequest => write!(f, "Bad Request"),
            AppError::Duplicate => write!(f, "Duplicate"),
            AppError::InternalServerError => write!(f, "Internal Server Error"),
            AppError::ServiceUnavailable => write!(f, "Service Unavailable"),
        }
    }
}

pub async fn handle_recover(err: Rejection) -> Result<impl warp::Reply, Infallible> {
    let (code, message) = if let Some(error) = err.find::<AppError>() {
        match error {
            AppError::NotFound => (StatusCode::NOT_FOUND, error.to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, error.to_string()),
            AppError::InvalidToken => (StatusCode::UNAUTHORIZED, error.to_string()),
            AppError::BadRequest => (StatusCode::BAD_REQUEST, error.to_string()),
            AppError::Duplicate => (StatusCode::CONFLICT, error.to_string()),
            AppError::InternalServerError => (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()),
            AppError::ServiceUnavailable => (StatusCode::SERVICE_UNAVAILABLE, error.to_string()),
        }
    } else if let Some(error) = err.find::<BodyDeserializeError>() {
        if cfg!(debug_assertions) {
            (StatusCode::UNPROCESSABLE_ENTITY, error.to_string())
        } else {
            (StatusCode::BAD_REQUEST, "Bad Request".to_string())
        }
    } else if err.find::<warp::reject::MethodNotAllowed>().is_some() {
        (StatusCode::NOT_FOUND, "".to_string())
    } else if err.find::<warp::reject::UnsupportedMediaType>().is_some() {
        (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Unsupported media type".to_string(),
        )
    } else if err.is_not_found() {
        (StatusCode::NOT_FOUND, "".to_string())
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error".to_string(),
        )
    };

    Ok(warp::reply::with_status(message, code))
}
