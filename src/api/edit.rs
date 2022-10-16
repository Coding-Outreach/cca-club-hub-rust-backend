use crate::{
    auth::{ExtractAuth},
    error::{AppError, AppResult},
    models::{Category, ClubCategory},
    schema::*,
    DbPool,
};
use axum::{
    extract::{Path},
    http::{StatusCode},
    routing::post,
    Extension, Json, Router,
};
use diesel::{delete, insert_into, update, AsChangeset, ExpressionMethods};
use diesel_async::RunQueryDsl;
use serde::{Deserialize};
use std::collections::{HashMap, HashSet};

#[derive(AsChangeset)]
#[diesel(table_name = club_socials)]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClubSocialRequest {
    website: Option<String>,
    google_classroom: Option<String>,
    discord: Option<String>,
    instagram: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClubRequest {
    #[allow(dead_code)]
    id: Option<String>,
    email: Option<String>,
    club_name: Option<String>,
    description: Option<String>,
    meet_time: Option<String>,
    profile_picture_url: Option<String>,
    categories: Vec<String>,
    socials: Option<ClubSocialRequest>,
}

#[derive(AsChangeset)]
#[diesel(table_name = clubs)]
struct ClubEdit {
    email: Option<String>,
    club_name: Option<String>,
    description: Option<String>,
    meet_time: Option<String>,
    profile_picture_url: Option<String>, 
}

async fn edit_club(
    Extension(pool): Extension<DbPool>,
    Json(req): Json<ClubRequest>,
    Path(club_id): Path<i32>,
    ExtractAuth(auth): ExtractAuth,
) -> AppResult<Json<()>> {
    auth.is_authorized(club_id)?;

    let conn = &mut pool.get().await?;

    update(clubs::table)
        .set(ClubEdit {
            club_name: req.club_name,
            email: req.email,
            meet_time: req.meet_time,
            description: req.description,
            profile_picture_url: req.profile_picture_url,
        })
        .execute(conn)
        .await?;

    let all_categories = HashMap::<_, _>::from_iter(
        categories::table
            .load::<Category>(conn)
            .await?
            .into_iter()
            .map(|c| (c.category_name, c.id)),
    );

    let new_categories: Vec<String> = req.categories.into_iter().collect::<HashSet<String>>().into_iter().collect();

    for category in &new_categories {
        if !all_categories.contains_key(category) {
            return Err(AppError::from(StatusCode::BAD_REQUEST, "invalid category"));
        }
    }

    let mut club_categories_id = delete(club_categories::table)
        .filter(club_categories::club_id.eq(club_id))
        .get_results::<ClubCategory>(conn)
        .await?
        .into_iter()
        .map(|c| c.id)
        .max()
        .unwrap_or(0)
        + 1;

    let new_club_categories: Vec<ClubCategory> = new_categories.into_iter().map(|name| {
        let id = club_categories_id;
        club_categories_id += 1;
        ClubCategory {
            id,
            club_id,
            category_id: *all_categories.get(&name).unwrap(),
        }
    }).collect();

    insert_into(club_categories::table)
        .values(new_club_categories)
        .execute(conn).await?;

    if let Some(club_social) = req.socials {
        update(club_socials::table)
            .filter(club_socials::club_id.eq(club_id))
            .set(club_social)
            .execute(conn)
            .await?;
    }

    Ok(Json(()))
}

pub fn app() -> Router {
    Router::new().route("/:club_id", post(edit_club))
}
