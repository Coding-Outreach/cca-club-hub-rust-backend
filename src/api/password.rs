use crate::{
    auth,
    email::{self, EMAIL_ADDRESS, FRONTEND_HOST},
    error::{AppError, AppResult},
    models::Club,
    schema::*,
    DbPool,
};
use axum::{
    extract::Path,
    http::StatusCode,
    routing::{get, post},
    Extension, Json, Router,
};
use diesel::{update, ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use lettre::{message::Mailbox, Address, Message};
use nanoid::nanoid;
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

#[allow(dead_code)]
type ResetState = Arc<Mutex<Resets>>;

#[derive(Default)]
pub struct Resets(pub HashMap<String, (Instant, Duration, i32)>);

impl Resets {
    #[allow(dead_code)]
    async fn clean(&mut self) {
        self.0
            .retain(|_, (instant, dur, _)| instant.elapsed() < *dur);
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PwdRequest {
    email: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NewPwdRequest {
    password: String,
}

// 1 hour
const RESET_ALLOWED_TIME: Duration = Duration::from_secs(60 * 60);

async fn password_request(
    Extension(pool): Extension<DbPool>,
    Extension(resets): Extension<Arc<Mutex<Resets>>>,
    Json(req): Json<PwdRequest>,
) -> AppResult<()> {
    let conn = &mut pool.get().await?;

    let Some(club) = clubs::table
        .filter(clubs::email.eq(req.email))
        .first::<Club>(conn)
        .await
        .optional()?
    else {
        return Err(AppError::from(
            StatusCode::NOT_FOUND,
            "could not find matching club",
        ));
    };

    let uid = nanoid!();
    let link = format!("{}/password/{}", *FRONTEND_HOST, uid);
    let body = format!(
        r"Hi {},

We have received a request to change your CCA Club Hub password. To reset your password, please click the below link within the next {} minutes (or paste it into your browser if clicking is not working):

{link}

If you did not request this password reset you can disregard this message and your password will remain unchanged.

Thanks,
The CCA Club Hub Team.",
        club.username,
        RESET_ALLOWED_TIME.as_secs() / 60
    );

    let destination_address = club
        .email
        .parse::<Address>()
        .map_err(|_| AppError::from(StatusCode::BAD_REQUEST, "invalid email"))?;

    let email = Message::builder()
        .from(Mailbox::new(
            Some("apathetic programmers".to_string()),
            EMAIL_ADDRESS.clone(),
        ))
        .to(Mailbox::new(Some(club.username), destination_address))
        .subject("CCA Club Hub Password Reset")
        .body(body)
        .unwrap();

    match email::send(email).await {
        Ok(_) => {
            let mut resets = resets.lock().await;
            resets
                .0
                .insert(uid, (Instant::now(), RESET_ALLOWED_TIME, club.id));
            Ok(())
        }
        Err(_) => Err(AppError::from(
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to send email",
        )),
    }
}

async fn password_reset(
    Extension(pool): Extension<DbPool>,
    Extension(resets): Extension<Arc<Mutex<Resets>>>,
    Path(uid): Path<String>,
    Json(req): Json<NewPwdRequest>,
) -> AppResult<()> {
    let mut resets = resets.lock().await;

    let Some((instant, allowed_time, club_id)) = resets.0.get(&uid) else {
        return Err(AppError::from(
            StatusCode::UNAUTHORIZED,
            "invalid password reset url",
        ));
    };

    if instant.elapsed() > *allowed_time {
        resets.0.remove(&uid);
        return Err(AppError::from(
            StatusCode::UNAUTHORIZED,
            "password reset expired",
        ));
    }

    let conn = &mut pool.get().await?;

    update(clubs::table.find(club_id))
        .set(clubs::password_hash.eq(auth::hash_password(req.password)?))
        .execute(conn)
        .await?;

    resets.0.remove(&uid);

    Ok(())
}

async fn check_uid(
    Extension(resets): Extension<Arc<Mutex<Resets>>>,
    Path(uid): Path<String>,
) -> AppResult<()> {
    resets.lock().await.0.get(&uid).map_or(
        Err(AppError::from(
            StatusCode::BAD_REQUEST,
            "invalid password reset url",
        )),
        |_| Ok(()),
    )
}

pub fn app() -> Router {
    // TODO we need to fix this. we moved the reset state to the api layer so we need to do something about this polling behaviour
    // let shared_resets = ResetState::default();
    // let uids = shared_resets.clone();
    // tokio::task::spawn(async move {
    //     let mut interval = interval(Duration::from_secs(24 * 60 * 60));

    //     loop {
    //         interval.tick().await;
    //         uids.lock().await.clean().await;
    //     }
    // });

    Router::new()
        .route("/reset", post(password_request))
        .route("/:uid", post(password_reset))
        .route("/check/:uid", get(check_uid))
}
