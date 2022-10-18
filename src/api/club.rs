use crate::{
    error::{AppError, AppResult},
    models::{Category, Club, ClubCategory, ClubSocial},
    schema::*,
    DbPool,
};
use axum::{extract::Path, http::StatusCode, routing::get, Extension, Json, Router};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::Serialize;
use std::collections::HashMap;

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

impl ClubResponse {
    fn from(
        club: Club,
        all_categories: &HashMap<i32, String>,
        category_ids: impl IntoIterator<Item = i32>,
        socials: Option<ClubSocial>,
    ) -> AppResult<ClubResponse> {
        Ok(ClubResponse {
            id: club.username.to_string(),
            email: club.email.clone(),
            club_name: club.club_name,
            description: club.description,
            about: club.about,
            meet_time: club.meet_time,
            profile_picture_url: club.profile_picture_url,
            featured: club.featured,
            categories: category_ids
                .into_iter()
                .map(|c| all_categories.get(&c).cloned())
                .collect::<Option<_>>()
                .ok_or_else(|| {
                    anyhow::anyhow!("database is in an invalid state: invalid category_id")
                })?,
            socials: ClubSocialResponse::from(club.email, socials),
        })
    }
}

async fn load_clubs(
    conn: &mut diesel_async::pg::AsyncPgConnection,
    clubs: Vec<Club>,
) -> AppResult<Vec<ClubResponse>> {
    let all_categories = HashMap::<_, _>::from_iter(
        categories::table
            .load::<Category>(conn)
            .await?
            .into_iter()
            .map(|c| (c.id, c.category_name)),
    );
    let club_category_ids = ClubCategory::belonging_to(&clubs)
        .load::<ClubCategory>(conn)
        .await?
        .grouped_by(&clubs);

    let club_socials = ClubSocial::belonging_to(&clubs)
        .load::<ClubSocial>(conn)
        .await?
        .grouped_by(&clubs);

    clubs
        .into_iter()
        .zip(club_category_ids)
        .zip(club_socials)
        .map(|((club, category_ids), mut socials)| {
            ClubResponse::from(
                club,
                &all_categories,
                category_ids.into_iter().map(|c| c.category_id),
                socials.pop(),
            )
        })
        .collect::<Result<_, _>>()
}

async fn list(Extension(pool): Extension<DbPool>) -> AppResult<Json<Vec<ClubResponse>>> {
    let conn = &mut pool.get().await?;

    let clubs = clubs::table.load::<Club>(conn).await?;

    Ok(Json(load_clubs(conn, clubs).await?))
}

async fn list_featured(Extension(pool): Extension<DbPool>) -> AppResult<Json<Vec<ClubResponse>>> {
    let conn = &mut pool.get().await?;

    let clubs = clubs::table
        .filter(clubs::featured.eq(true))
        .load::<Club>(conn)
        .await?;

    Ok(Json(load_clubs(conn, clubs).await?))
}

async fn info(
    Extension(pool): Extension<DbPool>,
    Path(club_id): Path<String>,
) -> AppResult<Json<ClubResponse>> {
    let conn = &mut pool.get().await?;

    let club = clubs::table
        .filter(clubs::username.eq(club_id))
        .first::<Club>(conn)
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
