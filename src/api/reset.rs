use crate::{
    auth,
    error::{AppError, AppResult},
    models::Club,
    schema::*,
    DbPool,
};
use axum::{extract::Path, http::StatusCode, routing::post, Extension, Json, Router};
use diesel::{query_dsl::methods::FilterDsl, update, ExpressionMethods, OptionalExtension};
use diesel_async::RunQueryDsl;
use lettre::{
    transport::smtp::authentication::Credentials, AsyncSmtpTransport, AsyncTransport, Message,
    Tokio1Executor,
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;

struct Resets(HashMap<String, (Instant, i32)>);

lazy_static::lazy_static! {
    static ref EMAIL_PWD: String = std::env::var("EMAIL_PWD").expect("email access password must be set for password reset responses");
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

async fn password_request(
    Extension(pool): Extension<DbPool>,
    Extension(resets): Extension<Arc<Mutex<Resets>>>,
    Json(req): Json<PwdRequest>,
) -> AppResult<()> {
    let conn = &mut pool.get().await?;

    if let Some(club) = clubs::table
        .filter(clubs::email.eq(req.email))
        .first::<Club>(conn)
        .await
        .optional()?
    {
        let mut resets = resets.lock().await;
        resets
            .0
            .insert(club.username.clone(), (Instant::now(), club.id));
        std::mem::drop(resets);

        let email = Message::builder()
            .from("server overlords <cca.club.hub@gmail.com>".parse().unwrap())
            .reply_to("server overlords <cca.club.hub@gmail.com>".parse().unwrap())
            .to(format!("{} <{}>", club.username, club.email)
                .parse()
                .unwrap())
            .subject("CCA Club Hub Password Reset")
            .body(format!("http://localhost:8080/reset/{}", club.username))
            .unwrap();

        let creds = Credentials::new("cca.club.hub@gmail.com".to_string(), EMAIL_PWD.to_string());

        // Open a remote connection to gmail
        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.gmail.com")
                .unwrap()
                .credentials(creds)
                .build();

        // Send the email
        match mailer.send(email).await {
            Ok(_) => Ok(()),
            Err(_) => Err(AppError::from(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to send email",
            )),
        }
    } else {
        Err(AppError::from(
            StatusCode::NOT_FOUND,
            "could not find matching club",
        ))
    }
}

async fn password_reset(
    Extension(pool): Extension<DbPool>,
    Extension(resets): Extension<Arc<Mutex<Resets>>>,
    Path(id): Path<String>,
    Json(req): Json<NewPwdRequest>,
) -> AppResult<()> {
    let mut resets = resets.lock().await;
    if let Some((instant, club_id)) = resets.0.get(&id) {
        if instant.elapsed() > Duration::from_secs(60 * 60) {
            resets.0.remove(&id);
            return Err(AppError::from(
                StatusCode::UNAUTHORIZED,
                "password reset expired",
            ));
        }

        let conn = &mut pool.get().await?;

        update(clubs::table)
            .filter(clubs::id.eq(club_id))
            .set(clubs::password_hash.eq(auth::hash_password(req.password)?))
            .execute(conn)
            .await?;

        resets.0.remove(&id);

        Ok(())
    } else {
        Err(AppError::from(
            StatusCode::UNAUTHORIZED,
            "invalid password reset url",
        ))
    }
}

pub fn app() -> Router {
    let shared_resets = Arc::new(Mutex::new(Resets(HashMap::new())));

    Router::new()
        .route("/password", post(password_request))
        .route("/:id", post(password_reset))
        .layer(Extension(shared_resets))
}
