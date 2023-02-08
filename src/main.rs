use axum::{http::Method, Extension};
use cca_club_hub::{auth::ensure_jwt_secret_is_valid, connect_to_db};
use envconfig::Envconfig;
use tower_http::cors::{Any, CorsLayer};

#[derive(Envconfig)]
struct Config {
    #[envconfig(from = "DATABASE_URL")]
    pub db_url: String,
    #[envconfig(from = "PORT", default = "8080")]
    pub port: u16,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let config = Config::init_from_env().unwrap();
    ensure_jwt_secret_is_valid();

    let pool = connect_to_db(&config.db_url);
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT])
        .allow_headers(Any)
        .allow_origin(Any);
    let app = cca_club_hub::app().layer(Extension(pool)).layer(cors);

    axum::Server::bind(&([0, 0, 0, 0], config.port).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
