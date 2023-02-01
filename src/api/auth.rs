use crate::{
    auth,
    error::{AppError, AppResult},
    models::Club,
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
    pub description: String,
    pub meet_time: String,
}

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
            // expires after one day
            token: auth::generate_jwt(club, Duration::from_secs(24 * 60 * 60))?,
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
        description: String,
        about: String,
        meet_time: String,
        profile_picture_url: String,
        featured: bool,
    }

    #[derive(Insertable)]
    #[diesel(table_name = club_socials)]
    struct NewClubSocial {
        club_id: i32,
    }

    let conn = &mut pool.get().await?;

    let new_club = diesel::insert_into(clubs::table)
        .values(NewClub {
            username: req.username,
            email: req.email,
            password_hash: auth::hash_password(req.password)?,
            club_name: req.name,
            description: req.description,
            about: "".to_string(),
            meet_time: req.meet_time,
            profile_picture_url: DEFAULT_PROFILE_PICTURE_URL.to_string(),
            featured: false,
        })
        .on_conflict(clubs::username)
        .do_nothing()
        .get_result::<Club>(conn)
        .await
        .optional()?;

    let Some(new_club) = new_club else {
        return Err(AppError::from(
            StatusCode::CONFLICT,
            "username has been taken",
        ));
    };

    diesel::insert_into(club_socials::table)
        .values(NewClubSocial {
            club_id: new_club.id,
        })
        .execute(conn)
        .await?;

    Ok(Json(ClubAuthorizedResponse::from_club(&new_club)?))
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
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}
