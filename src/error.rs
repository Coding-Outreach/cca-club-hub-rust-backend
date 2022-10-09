use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::borrow::Cow;

pub enum AppError {
    InternalServerError(anyhow::Error),
    ResponseStatusError(StatusCode, Cow<'static, str>),
}

pub type AppResult<T> = Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct AppErrorResponse {
            status: u16,
            message: Cow<'static, str>,
        }

        match self {
            AppError::InternalServerError(_err) => {
                // TODO: log error
                AppError::from(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
                    .into_response()
            }
            AppError::ResponseStatusError(code, s) => (
                code,
                Json(AppErrorResponse {
                    status: code.as_u16(),
                    message: s,
                }),
            )
                .into_response(),
        }
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(e: E) -> AppError {
        AppError::InternalServerError(e.into())
    }
}

impl AppError {
    pub fn from(code: StatusCode, s: impl Into<Cow<'static, str>>) -> AppError {
        AppError::ResponseStatusError(code, s.into())
    }
}
