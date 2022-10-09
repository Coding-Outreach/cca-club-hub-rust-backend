use crate::{
    auth,
    error::{AppError, AppResult},
    models::Club,
    DbPool,
};
use axum::{http::StatusCode, routing::post, Extension, Json, Router};
use diesel::{dsl::max, prelude::*};
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Deserialize)]
struct ClubRegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    pub name: String,
    pub description: Option<String>,
    pub meet_time: Option<String>,
}

#[derive(Deserialize)]
struct ClubLoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClubAuthorizedResponse {
    pub club_id: i32,
    pub jwt_token: String,
}

impl ClubAuthorizedResponse {
    fn from_club_id(club_id: i32) -> anyhow::Result<ClubAuthorizedResponse> {
        Ok(ClubAuthorizedResponse {
            club_id,
            // expires after one day
            jwt_token: auth::generate_jwt(club_id, Duration::from_secs(24 * 60 * 60))?,
        })
    }
}

// TODO: email users after registering
async fn register(
    Extension(pool): Extension<DbPool>,
    Json(req): Json<ClubRegisterRequest>,
) -> AppResult<Json<ClubAuthorizedResponse>> {
    use crate::schema::clubs::dsl::*;

    let conn = &mut pool.get().await?;
    let new_id: i32 = clubs
        .select(max(id))
        .first::<Option<i32>>(conn)
        .await?
        .unwrap_or(0)
        + 1;

    diesel::insert_into(clubs)
        .values(Club {
            id: new_id,
            username: req.username,
            email: req.email,
            password_hash: auth::hash_password(req.password)?,
            club_name: req.name,
            description: req.description,
            meet_time: req.meet_time,
            profile_picture_url: None,
            featured: false,
        })
        .execute(conn)
        .await?;

    Ok(Json(ClubAuthorizedResponse::from_club_id(new_id)?))
}

async fn login(
    Extension(pool): Extension<DbPool>,
    Json(req): Json<ClubLoginRequest>,
) -> AppResult<Json<ClubAuthorizedResponse>> {
    use crate::schema::clubs::dsl::*;

    let conn = &mut pool.get().await?;

    if let Some((club_id, club_password_hash)) = clubs
        .select((id, password_hash))
        .filter(username.eq(req.username))
        .first::<(i32, String)>(conn)
        .await
        .optional()?
    {
        if auth::verify_password(req.password, club_password_hash)? {
            return Ok(Json(ClubAuthorizedResponse::from_club_id(club_id)?));
        }
    }
    Err(AppError::from(
        StatusCode::UNAUTHORIZED,
        "invalid email or password",
    ))
}

pub fn app() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}
