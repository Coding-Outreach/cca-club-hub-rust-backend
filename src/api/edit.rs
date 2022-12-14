use crate::{
    auth::Auth,
    error::{AppError, AppResult},
    models::{Category, ClubCategory},
    schema::*,
    DbPool,
};
use axum::{extract::Path, http::StatusCode, routing::post, Extension, Json, Router};
use diesel::{delete, insert_into, prelude::*, update, AsChangeset, ExpressionMethods};
use diesel_async::RunQueryDsl;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(AsChangeset)]
#[diesel(treat_none_as_null = true)]
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
    email: String,
    club_name: String,
    description: Option<String>,
    about: Option<String>,
    meet_time: Option<String>,
    profile_picture_url: String,
    categories: Vec<String>,
    socials: ClubSocialRequest,
}

#[derive(AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[diesel(table_name = clubs)]
struct ClubEdit {
    email: String,
    club_name: String,
    description: Option<String>,
    about: Option<String>,
    meet_time: Option<String>,
    profile_picture_url: String,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[diesel(table_name = club_categories)]
struct NewClubCategory {
    club_id: i32,
    category_id: i32,
}

async fn edit_club(
    Extension(pool): Extension<DbPool>,
    Json(req): Json<ClubRequest>,
    Path(club_id): Path<String>,
    auth: Auth,
) -> AppResult<()> {
    let club_id = auth.into_authorized(&club_id)?.club_db_id;

    let conn = &mut pool.get().await?;

    update(clubs::table)
        .set(ClubEdit {
            club_name: req.club_name,
            email: req.email,
            meet_time: req.meet_time,
            description: req.description,
            about: req.about,
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

    let new_categories: Vec<String> = req
        .categories
        .into_iter()
        .collect::<HashSet<String>>()
        .into_iter()
        .collect();

    for category in &new_categories {
        if !all_categories.contains_key(category) {
            return Err(AppError::from(StatusCode::BAD_REQUEST, "invalid category"));
        }
    }

    delete(club_categories::table)
        .filter(club_categories::club_id.eq(club_id))
        .get_results::<ClubCategory>(conn)
        .await?;

    let new_club_categories: Vec<NewClubCategory> = new_categories
        .into_iter()
        .map(|name| NewClubCategory {
            club_id,
            category_id: *all_categories.get(&name).unwrap(),
        })
        .collect();

    insert_into(club_categories::table)
        .values(new_club_categories)
        .execute(conn)
        .await?;

    update(club_socials::table)
        .filter(club_socials::club_id.eq(club_id))
        .set(req.socials)
        .execute(conn)
        .await?;

    Ok(())
}

pub fn app() -> Router {
    Router::new().route("/:club_id", post(edit_club))
}
