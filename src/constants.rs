pub const DEFAULT_JWT_EXPIRATION: i64 = 3600;
pub const DEFAULT_JWT_REFRESH_EXPIRATION: i64 = 3600*24*7;

pub const REFRESH_API_PATH: &str = "/api/auth/refresh";
pub const UNPROTECTED_API_PATHS: [&str; 3] = [
    "/api/health",
    "/api/auth/signup",
    "/api/auth/login",
];