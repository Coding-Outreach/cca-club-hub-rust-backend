use axum::Router;
use deadpool::managed::Pool;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};

pub mod api;
pub mod auth;
pub mod email;
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
    Router::new().nest("/api", api::app())
}
