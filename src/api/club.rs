use crate::{
    error::{AppError, AppResult},
    models::{Category, Club, ClubCategory, ClubSocial},
    schema::*,
    DbPool,
};
use axum::{extract::Path, http::StatusCode, routing::get, Extension, Json, Router};
use diesel::{prelude::*, BelongingToDsl};
use diesel_async::RunQueryDsl;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClubSocialResponse {
    website: String,
    google_classroom: String,
    discord: String,
    instagram: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClubResponse {
    id: String,
    email: String,
    club_name: String,
    description: Option<String>,
    meet_time: Option<String>,
    profile_picture_url: Option<String>,
    featured: bool,
    categories: Vec<String>,
    socials: Vec<ClubSocialResponse>,
}

impl ClubResponse {
    fn from(
        club: Club,
        all_categories: &HashMap<i32, String>,
        category_ids: impl IntoIterator<Item = i32>,
        socials: impl IntoIterator<Item = ClubSocial>,
    ) -> AppResult<ClubResponse> {
        Ok(ClubResponse {
            id: club.id.to_string(),
            email: club.email,
            club_name: club.club_name,
            description: club.description,
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
            socials: socials
                .into_iter()
                .map(|s| ClubSocialResponse {
                    website: s.website,
                    google_classroom: s.google_classroom,
                    discord: s.discord,
                    instagram: s.instagram,
                })
                .collect(),
        })
    }
}

async fn list(Extension(pool): Extension<DbPool>) -> AppResult<Json<Vec<ClubResponse>>> {
    let conn = &mut pool.get().await?;

    let clubs = clubs::table.load::<Club>(conn).await?;

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

    Ok(Json(
        clubs
            .into_iter()
            .zip(club_category_ids)
            .zip(club_socials)
            .map(|((club, category_ids), socials)| {
                ClubResponse::from(
                    club,
                    &all_categories,
                    category_ids.into_iter().map(|c| c.category_id),
                    socials,
                )
            })
            .collect::<Result<_, _>>()?,
    ))
}

async fn info(
    Extension(pool): Extension<DbPool>,
    Path(club_id): Path<i32>,
) -> AppResult<Json<ClubResponse>> {
    let conn = &mut pool.get().await?;

    let club = clubs::table
        .find(club_id)
        .first::<Club>(conn)
        .await
        .optional()?
        .ok_or_else(|| AppError::from(StatusCode::NOT_FOUND, "the club does not exist"))?;

    let club_category_ids = ClubCategory::belonging_to(&club)
        .select(club_categories::id)
        .load::<i32>(conn)
        .await?;
    let all_categories = HashMap::<_, _>::from_iter(
        categories::table
            .filter(categories::dsl::id.eq_any(club_category_ids.clone()))
            .load::<Category>(conn)
            .await?
            .into_iter()
            .map(|c| (c.id, c.category_name)),
    );

    let club_socials = ClubSocial::belonging_to(&club)
        .load::<ClubSocial>(conn)
        .await?;

    Ok(Json(ClubResponse::from(
        club,
        &all_categories,
        club_category_ids,
        club_socials,
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
        .route("/info/:club_id", get(info))
        .route("/categories/list", get(list_categories))
}
