use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug)]
pub enum AppError {
    NotFound,
    BadRequest(String),
    Unauthorized,
    Forbidden,
    Internal(String),
}

impl From<worker::KvError> for AppError {
    fn from(err: worker::KvError) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<worker::Error> for AppError {
    fn from(err: worker::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

#[derive(Serialize)]
struct Err {
    msg: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                Json(Err {
                    msg: "not_found".into(),
                }),
            ),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, Json(Err { msg })),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(Err {
                    msg: "UNAUTHORIZED".into(),
                }),
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                Json(Err {
                    msg: "FORBIDDEN".into(),
                }),
            ),
            AppError::Internal(_err) => (
                // log the err or put into a tracing span!
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Err {
                    msg: "INTERNAL SERVER ERROR".into(),
                }),
            ),
        }
        .into_response()
    }
}
