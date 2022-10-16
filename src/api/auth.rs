use crate::{
    auth,
    error::{AppError, AppResult},
    schema::*,
    DbPool,
};
use axum::{http::StatusCode, routing::post, Extension, Json, Router};
use diesel::prelude::*;
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

// TODO: default profile picture url
const DEFAULT_PROFILE_PICTURE_URL: &str = "";

// TODO: email users after registering
async fn register(
    Extension(pool): Extension<DbPool>,
    Json(req): Json<ClubRegisterRequest>,
) -> AppResult<Json<ClubAuthorizedResponse>> {
    #[derive(Insertable)]
    #[diesel(table_name = clubs)]
    struct NewClub {
        username: String,
        email: String,
        password_hash: String,
        club_name: String,
        description: Option<String>,
        meet_time: Option<String>,
        profile_picture_url: String,
        featured: bool,
    }

    let conn = &mut pool.get().await?;

    let new_id = diesel::insert_into(clubs::table)
        .values(NewClub {
            username: req.username,
            email: req.email,
            password_hash: auth::hash_password(req.password)?,
            club_name: req.name,
            description: req.description,
            meet_time: req.meet_time,
            profile_picture_url: DEFAULT_PROFILE_PICTURE_URL.to_string(),
            featured: false,
        })
        .on_conflict(clubs::username)
        .do_nothing()
        .returning(clubs::id)
        .get_result(conn)
        .await
        .optional()?;

    if let Some(new_id) = new_id {
        Ok(Json(ClubAuthorizedResponse::from_club_id(new_id)?))
    } else {
        Err(AppError::from(
            StatusCode::CONFLICT,
            "username has been taken",
        ))
    }
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
        "invalid username or password",
    ))
}

pub fn app() -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}
