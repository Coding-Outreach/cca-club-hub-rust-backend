use crate::{
    auth::{validate_jwt, Claims},
    error::{AppError, AppResult},
    models::{Category, ClubCategory},
    schema::*,
    DbPool,
};
use axum::{
    extract::Path,
    http::{header::HeaderMap, StatusCode},
    routing::post,
    Extension, Json, Router,
};
use diesel::{delete, dsl::max, insert_into, prelude::*, update, AsChangeset, ExpressionMethods};
use diesel_async::RunQueryDsl;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClubSocialRequest {
    website: Option<String>,
    google_classroom: Option<String>,
    discord: Option<String>,
    instagram: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClubRequest {
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
) -> AppResult<StatusCode> {
    auth::is_authorized(club_id)?;

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

    let new_categories = {
        let seen = HashSet::new();
        let new_categories = req.categories;
        new_categories.retain(|name| seen.insert(name));
        new_categories
    };

    for category in &new_categories {
        if !all_categories.contains_key(category) {
            return Err(AppError::from(StatusCode::UNAUTHORIZED, "invalid category"));
        }
    }

    let mut club_categories_id = delete(club_categories::table)
        .filter(club_categories::club_id.eq(club_id))
        .get_results(conn)
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

    let club_socials_id = delete(club_socials::table)
        .filter(club_socials::club_id.eq(club_id))
        .get_results(conn)
        .await?
        .into_iter()
        .map(|c| c.id)
        .max()
        .unwrap_or(0)
        + 1;

    Err(AppError::from(StatusCode::ACCEPTED, "an error occured"))
}

pub fn app() -> Router {
    Router::new().route("/:club_id", post(edit_club))
}
