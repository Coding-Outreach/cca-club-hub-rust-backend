use crate::{
    auth,
    error::{AppError, AppResult},
    models::Club,
    DbPool,
};
use axum::{http::StatusCode, routing::post, Extension, Json, Router};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Deserialize)]
struct ClubLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClubAuthorizedResponse {
    pub token: String,
}

impl ClubAuthorizedResponse {
    fn from_club(club: &Club) -> anyhow::Result<ClubAuthorizedResponse> {
        // expires after one day
        Ok(ClubAuthorizedResponse {
            token: auth::generate_jwt(club, Duration::from_secs(24 * 60 * 60))?,
        })
    }
}

async fn login(
    Extension(pool): Extension<DbPool>,
    Json(req): Json<ClubLoginRequest>,
) -> AppResult<Json<ClubAuthorizedResponse>> {
    use crate::schema::clubs::dsl::*;

    let conn = &mut pool.get().await?;

    if let Some(club) = clubs
        .filter(username.eq(req.username))
        .first::<Club>(conn)
        .await
        .optional()?
    {
        if auth::verify_password(req.password, &club.password_hash)? {
            return Ok(Json(ClubAuthorizedResponse::from_club(&club)?));
        }
    }
    Err(AppError::from(
        StatusCode::UNAUTHORIZED,
        "invalid username or password",
    ))
}

pub fn app() -> Router {
    Router::new().route("/login", post(login))
}
