pub const DEFAULT_JWT_EXPIRATION: i64 = 3600;
pub const DEFAULT_JWT_REFRESH_EXPIRATION: i64 = 3600 * 24 * 7;
pub const DEFAULT_OAUTH_CONFIG_PATH: &str = "/config/oauth_config.toml";

pub const REFRESH_API_PATH: &str = "/api/auth/refresh";
pub const OAUTH_GET_API_PATH: &str = "/api/auth/oauth2/authorize";
pub const UNPROTECTED_API_PATHS: [&str; 4] = [
    "/api/health",
    "/api/auth/signup",
    "/api/auth/login",
    "/api/reviews/public_key",
];

pub const REVIEWS_SIGNING_PUBLIC_KEY_NAME: &str = "reviews_public_key.pem";
pub const REVIEWS_SIGNING_PRIVATE_KEY_NAME: &str = "reviews_private_key.pem";
