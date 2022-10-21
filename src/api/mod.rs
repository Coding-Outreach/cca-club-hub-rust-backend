use axum::Router;

pub mod auth;
pub mod club;
pub mod edit;
pub mod password;

pub fn app() -> Router {
    Router::new()
        .nest("/auth", auth::app())
        .nest("/edit", edit::app())
        .nest("/club", club::app())
        .nest("/password", password::app())
}
