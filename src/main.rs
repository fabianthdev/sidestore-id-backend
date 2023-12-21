mod api;
mod auth;
mod config;
mod constants;
mod db;
mod middlewares;
mod services;
mod errors;
mod util;

use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use ed25519_dalek::SigningKey;
use log::info;

use crate::config::Config;
use crate::db::Pool;
use crate::util::review_signing::create_or_load_review_signing_key;


pub struct AppState {
    db: Pool,
    env: Config,
    review_signing_key: SigningKey,
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().expect("Failed to load .env file");
    env_logger::init();

    let config = Config::init();
    let app_url = format!("{}:{}", &config.host, &config.port);

    let pool = db::create_pool(&config.database_url);
    db::run_migration(&mut pool.get().unwrap());

    let review_signing_key = match create_or_load_review_signing_key(&config) {
        Ok(review_signing_key) => review_signing_key,
        Err(e) => {
            panic!("Failed to load review signing key: {}", e);
        }
    };

    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    // .allowed_origin(&config.cors_origin)
                    .allow_any_origin()
                    // .send_wildcard()
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec![
                        actix_web::http::header::AUTHORIZATION,
                        actix_web::http::header::CONTENT_TYPE,
                        actix_web::http::header::ACCEPT
                    ])
                    .supports_credentials()
                    .max_age(3600)
            )
            .wrap(Logger::default())
            .app_data(web::Data::new(AppState {
                db: pool.clone(),
                env: config.clone(),
                review_signing_key: review_signing_key.clone(),
            }))
            .wrap(actix_web::middleware::Logger::default())
            .configure(config::app::config_services)
    })
    .bind(&app_url)?
    .run();

    info!("Server running at http://{}", &app_url);

    server.await
}