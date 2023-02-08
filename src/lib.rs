use std::io;

use axum::{http::StatusCode, routing::get_service, Router};
use deadpool::managed::Pool;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use tower_http::services::ServeDir;

pub mod api;
pub mod auth;
pub mod error;
pub mod models;
pub mod schema;

pub type DbPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

pub fn connect_to_db(db_url: &str) -> Pool<AsyncDieselConnectionManager<AsyncPgConnection>> {
    let db_config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(db_url);
    Pool::builder(db_config)
        .build()
        .expect("failed to build database pool")
}

pub fn app() -> Router {
    let serve = get_service(ServeDir::new("assets")).handle_error(handle_error);
    Router::new()
        .nest("/api", api::app())
        .nest("/assets", serve)
}

async fn handle_error(_: io::Error) -> error::AppError {
    error::AppError::from(StatusCode::INTERNAL_SERVER_ERROR, "failed to fetch asset")
}
