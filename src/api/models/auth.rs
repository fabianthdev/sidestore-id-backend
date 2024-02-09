use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

use crate::db::models::user::User;

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, ToResponse)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub profile: User,
}

#[derive(Deserialize, ToSchema)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
}

pub type SignupResponse = LoginResponse;
