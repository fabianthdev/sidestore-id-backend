use crate::constants::{DEFAULT_JWT_EXPIRATION, DEFAULT_JWT_REFRESH_EXPIRATION, DEFAULT_OAUTH_CONFIG_PATH};

pub mod app;
pub mod db;

#[derive(Debug, Clone)]
pub struct Config {
    // pub secret_key: String,
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub jwt_issuer: String,
    pub jwt_expiration: i64,
    pub jwt_refresh_expiration: i64,
    pub cors_origin: String,
    pub public_url: String,
    pub database_url: String,
    pub storage_path: String,
    pub oauth_config_path: String,
}

impl Config {
    pub fn init() -> Config {
        // let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY must be set");
        let host = std::env::var("HOST").expect("HOST must be set");
        let port = match std::env::var("PORT").expect("PORT must be set").parse::<u16>() {
            Ok(val) => val,
            Err(_) => panic!("PORT must be an integer"),
        };
        let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let jwt_issuer = std::env::var("JWT_ISSUER").expect("JWT_ISSUER must be set");
        let jwt_expiration = match std::env::var("JWT_EXPIRATION") {
            Ok(val) => val.parse::<i64>().expect("JWT_EXPIRATION must be an integer"),
            Err(_) => DEFAULT_JWT_EXPIRATION,
        };
        let jwt_refresh_expiration = match std::env::var("JWT_REFRESH_EXPIRATION") {
            Ok(val) => val.parse::<i64>().expect("JWT_REFRESH_EXPIRATION must be an integer"),
            Err(_) => DEFAULT_JWT_REFRESH_EXPIRATION,
        };
        let cors_origin = std::env::var("CORS_ORIGIN").expect("CORS_ORIGIN must be set");
        let public_url = std::env::var("PUBLIC_URL").expect("PUBLIC_URL must be set");
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let storage_path = std::env::var("STORAGE_PATH").expect("STORAGE_PATH must be set");

        let oauth_config_path = std::env::var("OAUTH_CONFIG_PATH")
            .unwrap_or(DEFAULT_OAUTH_CONFIG_PATH.to_string());

        Config {
            // secret_key,
            host,
            port,
            jwt_secret,
            jwt_issuer,
            jwt_expiration,
            jwt_refresh_expiration,
            cors_origin,
            public_url,
            database_url,
            storage_path,
            oauth_config_path,
        }
    }
}
