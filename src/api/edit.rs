use crate::{
    auth::{validate_jwt, Claims},
    error::{AppError, AppResult},
};
use axum::{
    extract::Path,
    http::{header::HeaderMap, StatusCode},
    Json, Router,
};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SocialsRequest {
    social_name: String,
    social_link: String,
    social_id: i32,
}

fn edit_socials(Json(req): Json<SocialsRequest>, Path(club_id): Path<i32>) -> AppResult<StatusCode> {
    if let Some(claims) = headers
        .get("Authorization")
        .and_then(|token| token.to_str().ok())
        .and_then(|token| validate_jwt(token).ok())
    {

    } else {
        Err(AppError::from(
            StatusCode::UNAUTHORIZED,
            "invalid or missing token",
        ))
    }
}

pub fn app() -> Router {
    Router::new()
        .route("/socials/:club_id", post(edit_socials))
}
