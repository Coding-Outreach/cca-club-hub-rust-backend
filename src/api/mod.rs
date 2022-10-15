use axum::Router;

pub mod auth;
pub mod club;

pub fn app() -> Router {
    Router::new()
        .nest("/auth", auth::app())
        .nest("/club", club::app())
}
