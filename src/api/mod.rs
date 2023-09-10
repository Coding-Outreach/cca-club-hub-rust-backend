use std::{sync::Arc, collections::HashMap};

use axum::{Router, Extension};
use tokio::sync::Mutex;

use self::password::Resets;

pub mod admin;
pub mod auth;
pub mod club;
pub mod edit;
pub mod password;

pub fn app() -> Router {
    let shared_resets = Arc::new(Mutex::new(Resets(HashMap::new())));

    Router::new()
        .nest("/admin", admin::app())
        .nest("/auth", auth::app())
        .nest("/edit", edit::app())
        .nest("/club", club::app())
        .nest("/password", password::app())
        .layer(Extension(shared_resets))
}

pub const DEFAULT_PROFILE_PICTURE_URL: &str = "assets/default_pfp.png";
pub const DEFAULT_BANNER_URL: &str = "";
