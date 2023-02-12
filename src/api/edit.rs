use crate::{
    auth::Auth,
    error::{AppError, AppResult},
    models::{Category, ClubCategory},
    schema::*,
    DbPool,
};
use axum::{
    body::Bytes,
    headers::ContentType,
    http::StatusCode,
    routing::{post, put},
    Extension, Json, Router, TypedHeader,
};
use diesel::{delete, insert_into, prelude::*, update, AsChangeset, ExpressionMethods};
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use mime::Mime;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    env::current_dir,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

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
    club_name: String,
    description: String,
    about: String,
    meet_time: String,
    categories: Vec<String>,
    socials: ClubSocialRequest,
}

#[derive(AsChangeset)]
#[diesel(treat_none_as_null = true)]
#[diesel(table_name = clubs)]
struct ClubEdit {
    club_name: String,
    description: String,
    about: String,
    meet_time: String,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[diesel(table_name = club_categories)]
struct NewClubCategory {
    club_id: i32,
    category_id: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UploadPfpResponse {
    url: String,
}

lazy_static::lazy_static! {
    static ref CWD: PathBuf = current_dir().expect("could not get current working directory");
    static ref PFP_DIR: PathBuf = CWD.join("static/profile_pictures/");
}

async fn upload_pfp(
    Extension(pool): Extension<DbPool>,
    TypedHeader(content_type): TypedHeader<ContentType>,
    bytes: Bytes,
    Auth(auth): Auth,
) -> AppResult<Json<UploadPfpResponse>> {
    let club_id = auth.club_db_id;

    let conn = &mut pool.get().await?;
    let kind = infer::get(&bytes)
        .ok_or_else(|| AppError::from(StatusCode::BAD_REQUEST, "file type not recognized"))?;

    let mime: Mime = kind.mime_type().parse()?;

    if content_type != mime.clone().into() {
        return Err(AppError::from(
            StatusCode::BAD_REQUEST,
            "file type does not match Content-Type header",
        ));
    }

    if mime.type_() != mime::IMAGE {
        return Err(AppError::from(StatusCode::BAD_REQUEST, "file not an image"));
    }

    let mut hasher = Sha256::new();
    hasher.update(&bytes);

    let result = hasher.finalize();

    let file_name = format!("{:02x}.{}", result[..].iter().format(""), kind.extension());
    let mut path: PathBuf = ["assets", "pfp"].iter().collect();
    fs::create_dir_all(&path)?;

    path.push(&file_name);

    let mut file = File::create(&path)?;

    file.write_all(&bytes)?;

    let path_string = path
        .to_str()
        .ok_or_else(|| {
            AppError::from(
                StatusCode::INTERNAL_SERVER_ERROR,
                "can't turn path into string",
            )
        })?
        .to_string();

    update(clubs::table)
        .filter(clubs::id.eq(club_id))
        .set(clubs::profile_picture_url.eq(&path_string))
        .execute(conn)
        .await?;

    Ok(Json(UploadPfpResponse { url: path_string }))
}

async fn edit_club(
    Extension(pool): Extension<DbPool>,
    Json(req): Json<ClubRequest>,
    Auth(auth): Auth,
) -> AppResult<()> {
    let club_id = auth.club_db_id;

    let conn = &mut pool.get().await?;

    update(clubs::table)
        .set(ClubEdit {
            club_name: req.club_name,
            meet_time: req.meet_time,
            description: req.description,
            about: req.about,
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
    Router::new()
        .route("/info", post(edit_club))
        .route("/pfp", put(upload_pfp))
}
