use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct ResponseStatusError(StatusCode, Cow<'static, str>);

impl ResponseStatusError {
    pub fn from(code: StatusCode, s: impl Into<Cow<'static, str>>) -> Self {
        Self(code, s.into())
    }
}

impl IntoResponse for ResponseStatusError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct AppErrorResponse {
            status: u16,
            message: Cow<'static, str>,
        }

        (
            self.0,
            Json(AppErrorResponse {
                status: self.0.as_u16(),
                message: self.1,
            }),
        )
            .into_response()
    }
}

impl<S: Into<Cow<'static, str>>> From<(StatusCode, S)> for ResponseStatusError {
    fn from((code, s): (StatusCode, S)) -> Self {
        Self::from(code, s)
    }
}

pub enum AppError {
    InternalServerError(anyhow::Error),
    ResponseStatusError(ResponseStatusError),
}

pub type AppResult<T> = Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::InternalServerError(_err) => {
                // TODO: log error
                AppError::from(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
                    .into_response()
            }
            AppError::ResponseStatusError(rse) => rse.into_response(),
        }
    }
}

impl AppError {
    pub fn from(code: StatusCode, s: impl Into<Cow<'static, str>>) -> AppError {
        AppError::ResponseStatusError(ResponseStatusError::from(code, s))
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(e: E) -> AppError {
        AppError::InternalServerError(e.into())
    }
}

impl From<ResponseStatusError> for AppError {
    fn from(e: ResponseStatusError) -> AppError {
        AppError::ResponseStatusError(e)
    }
}
