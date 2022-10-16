use axum::Router;

pub mod auth;
pub mod edit;
pub mod club;

pub fn app() -> Router {
    Router::new()
        .nest("/auth", auth::app())
        .nest("/edit", edit::app())
        .nest("/club", club::app())
}
