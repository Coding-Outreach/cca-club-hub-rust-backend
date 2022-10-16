use axum::Router;

pub mod auth;
pub mod edit;

pub fn app() -> Router {
    Router::new()
        .nest("/auth", auth::app())
        .nest("/edit", edit::app())
}
