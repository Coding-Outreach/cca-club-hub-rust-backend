use crate::{
    error::{AppError, AppResult},
    models::{Category, Club, ClubCategory, ClubSocial},
    schema::*,
    DbPool,
};
use axum::{extract::Path, http::StatusCode, routing::get, Extension, Json, Router};
use diesel::prelude::*;
use diesel_async::{pg::AsyncPgConnection, RunQueryDsl};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClubSocialResponse {
    email: String,
    website: Option<String>,
    google_classroom: Option<String>,
    discord: Option<String>,
    instagram: Option<String>,
}

impl ClubSocialResponse {
    fn from(email: String, socials: Option<ClubSocial>) -> Self {
        if let Some(socials) = socials {
            Self {
                email,
                website: socials.website,
                google_classroom: socials.google_classroom,
                discord: socials.discord,
                instagram: socials.instagram,
            }
        } else {
            Self {
                email,
                website: None,
                google_classroom: None,
                discord: None,
                instagram: None,
            }
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClubResponse {
    id: String,
    email: String,
    club_name: String,
    description: Option<String>,
    about: Option<String>,
    meet_time: Option<String>,
    profile_picture_url: String,
    featured: bool,
    categories: Vec<String>,
    socials: ClubSocialResponse,
}

async fn load_clubs(
    conn: &mut AsyncPgConnection,
    clubs: Vec<(Club, Option<ClubSocial>)>,
) -> AppResult<Vec<ClubResponse>> {
    let categories = club_categories::table
        .inner_join(categories::table)
        .filter(club_categories::club_id.eq_any(clubs.iter().map(|c| c.0.id)))
        .load::<(ClubCategory, Category)>(conn)
        .await?
        .grouped_by(&clubs.iter().map(|c| &c.0).collect::<Vec<_>>());

    Ok(clubs
        .into_iter()
        .zip(categories)
        .map(|((club, socials), categories)| ClubResponse {
            id: club.username.to_string(),
            email: club.email.clone(),
            club_name: club.club_name,
            description: club.description,
            about: club.about,
            meet_time: club.meet_time,
            profile_picture_url: club.profile_picture_url,
            featured: club.featured,
            categories: categories.into_iter().map(|c| c.1.category_name).collect(),
            socials: ClubSocialResponse::from(club.email, socials),
        })
        .collect())
}

async fn list(Extension(pool): Extension<DbPool>) -> AppResult<Json<Vec<ClubResponse>>> {
    let conn = &mut pool.get().await?;

    let clubs = clubs::table
        .left_join(club_socials::table)
        .load(conn)
        .await?;

    Ok(Json(load_clubs(conn, clubs).await?))
}

async fn list_featured(Extension(pool): Extension<DbPool>) -> AppResult<Json<Vec<ClubResponse>>> {
    let conn = &mut pool.get().await?;

    let clubs = clubs::table
        .left_join(club_socials::table)
        .filter(clubs::featured.eq(true))
        .load(conn)
        .await?;

    Ok(Json(load_clubs(conn, clubs).await?))
}

async fn info(
    Extension(pool): Extension<DbPool>,
    Path(club_id): Path<String>,
) -> AppResult<Json<ClubResponse>> {
    let conn = &mut pool.get().await?;

    let club = clubs::table
        .left_join(club_socials::table)
        .filter(clubs::username.eq(club_id))
        .first(conn)
        .await
        .optional()?
        .ok_or_else(|| AppError::from(StatusCode::NOT_FOUND, "the club does not exist"))?;

    Ok(Json(load_clubs(conn, vec![club]).await?.pop().ok_or_else(
        || anyhow::anyhow!("`load_clubs` should return one club"),
    )?))
}

async fn list_categories(Extension(pool): Extension<DbPool>) -> AppResult<Json<Vec<String>>> {
    let conn = &mut pool.get().await?;

    Ok(Json(
        categories::table
            .load::<Category>(conn)
            .await?
            .into_iter()
            .map(|c| c.category_name)
            .collect(),
    ))
}

pub fn app() -> Router {
    Router::new()
        .route("/list", get(list))
        .route("/list/featured", get(list_featured))
        .route("/info/:club_id", get(info))
        .route("/categories/list", get(list_categories))
}
