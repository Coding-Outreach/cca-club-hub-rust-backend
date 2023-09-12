use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use super::{password::Resets, DEFAULT_BANNER_URL, DEFAULT_PROFILE_PICTURE_URL};
use crate::{
    auth::{self, AdminOnly},
    email::{self, EMAIL_ADDRESS, FRONTEND_HOST},
    error::{AppError, AppResult},
    models::Club,
    schema::*,
    DbPool,
};
use axum::{http::StatusCode, routing::post, Extension, Json, Router};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use lettre::{message::Mailbox, Address, Message};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

#[derive(Deserialize)]
struct ClubRegisterRequest {
    pub username: String,
    pub email: String,
    pub name: String,
    pub description: String,
    pub meet_time: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClubRegisterResponse {
    pub message: String,
}

impl ClubRegisterResponse {
    fn from_club(club: &Club) -> anyhow::Result<ClubRegisterResponse> {
        // expires after one day
        Ok(Self {
            // expires after one day
            message: format!(
                "Registered club {}, sent verification email to {}.",
                club.club_name, club.email
            ),
        })
    }
}

// TODO: email users after registering
async fn register(
    Extension(pool): Extension<DbPool>,
    Extension(resets): Extension<Arc<Mutex<Resets>>>,
    Json(req): Json<ClubRegisterRequest>,
    AdminOnly: AdminOnly,
) -> AppResult<Json<ClubRegisterResponse>> {
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
        banner_url: String,
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
            password_hash: auth::hash_password(rand::random::<[u8; 32]>())?,
            club_name: req.name,
            description: req.description,
            about: "".to_string(),
            meet_time: req.meet_time,
            profile_picture_url: DEFAULT_PROFILE_PICTURE_URL.to_string(),
            banner_url: DEFAULT_BANNER_URL.to_string(),
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
            "club already exists!",
        ));
    };

    diesel::insert_into(club_socials::table)
        .values(NewClubSocial {
            club_id: new_club.id,
        })
        .execute(conn)
        .await?;

    let uid = nanoid!();
    let link = format!("{}/password/{}", *FRONTEND_HOST, uid);
    let body = format!(
        r#"Hi {},

Welcome to the CCA Club Hub! We are so excited to have you here. To finish setting up your account, go ahead and open that link to set a password for your account, then login with the username "{}" and your new password!

{link}

This link will expire in 7 days (or when the server restarts), so if you need a new link, just use the "Forgot your password?" link on the login page to create a new link.

Thanks,
The CCA Club Hub Team."#,
        new_club.username, new_club.username,
    );

    let destination_address = new_club
        .email
        .parse::<Address>()
        .map_err(|_| AppError::from(StatusCode::BAD_REQUEST, "invalid email"))?;

    let email = Message::builder()
        .from(Mailbox::new(
            Some("apathetic programmers".to_string()),
            EMAIL_ADDRESS.clone(),
        ))
        .to(Mailbox::new(Some(new_club.username.clone()), destination_address))
        .subject("Welcome to the CCA Club Hub!")
        .body(body)
        .unwrap();

    match email::send(email).await {
        Ok(_) => {
            let mut resets = resets.lock().await;
            // 7 days
            resets.0.insert(
                uid,
                (
                    Instant::now(),
                    Duration::from_secs(60 * 60 * 24 * 7),
                    new_club.id,
                ),
            );
        }
        Err(_) => {
            return Err(AppError::from(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to send email",
            ))
        }
    }

    Ok(Json(ClubRegisterResponse::from_club(&new_club)?))
}

pub fn app() -> Router {
    Router::new().route("/register", post(register))
}
