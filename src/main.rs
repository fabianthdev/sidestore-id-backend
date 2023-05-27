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
use log::info;

use crate::config::Config;
use crate::db::Pool;


pub struct AppState {
    db: Pool,
    env: Config,
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().expect("Failed to load .env file");
    env_logger::init();

    let config = Config::init();
    let app_url = format!("{}:{}", &config.host, &config.port);

    let pool = db::create_pool(&config.database_url);
    db::run_migration(&mut pool.get().unwrap());

    let server = HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .send_wildcard()
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                    .allowed_headers(vec!["Authorization", "Content-Type", "Accept"])
                    .max_age(3600)
            )
            .wrap(Logger::default())
            .app_data(web::Data::new(AppState {
                db: pool.clone(),
                env: config.clone(),
            }))
            .wrap(actix_web::middleware::Logger::default())
            .configure(config::app::config_services)
    })
    .bind(&app_url)?
    .run();

    info!("Server running at http://{}", &app_url);

    server.await
}