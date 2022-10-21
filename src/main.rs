use axum::Extension;
use cca_club_hub::{auth::ensure_jwt_secret_is_valid, connect_to_db, email};
use envconfig::Envconfig;

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
    email::sanity_check().await;

    let pool = connect_to_db(&config.db_url);
    let app = cca_club_hub::app().layer(Extension(pool));

    axum::Server::bind(&([0, 0, 0, 0], config.port).into())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
