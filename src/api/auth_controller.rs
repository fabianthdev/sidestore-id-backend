use actix_web::{web, HttpResponse, Responder};

use crate::AppState;
use crate::db::models::user::User;
use crate::middlewares::auth::JwtMiddleware;
use crate::services::auth_service;
use crate::{db::models::user::UserDTO, errors::ServiceError};

use super::models::MessageResponse;
use super::models::auth::{LoginRequest, LoginResponse, SignupRequest, SignupResponse};


/// Registration endpoint for new users
#[utoipa::path(
    post,
    path = "/api/auth/signup",
    responses(
        (status = 200, response = SignupResponse),
    ),
)]
pub async fn signup(body: web::Json<SignupRequest>, data: web::Data<AppState>) -> Result<HttpResponse, ServiceError> {
    let user_dto = UserDTO { email: body.email.clone(), password: body.password.clone(), username: None };
    let (user, access_token, refresh_token) = auth_service::login(user_dto, &data.db, &data.env)?;

    Ok(HttpResponse::Ok().json(SignupResponse { 
        access_token,
        refresh_token,
        profile: user
    }))
}


/// Authentication endpoint for existing users
#[utoipa::path(
    post,
    path = "/api/auth/login",
    responses(
        (status = 200, response = LoginResponse),
    ),
)]
pub async fn login(body: web::Json<LoginRequest>, data: web::Data<AppState>) -> Result<HttpResponse, ServiceError> {
    let user_dto = UserDTO { email: body.email.clone(), password: body.password.clone(), username: None };
    let (user, access_token, refresh_token) = auth_service::login(user_dto, &data.db, &data.env)?;
    
    Ok(HttpResponse::Ok().json(LoginResponse {
        access_token,
        refresh_token,
        profile: user
    }))
}


/// Use a refresh token to get a new access token
#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    responses(
        (status = 200, response = LoginResponse),
    ),
)]
pub async fn refresh(data: web::Data<AppState>, jwt: JwtMiddleware) -> Result<HttpResponse, ServiceError> {
    let (user, access_token, refresh_token) = auth_service::refresh(&data.db, &data.env, jwt.user_id)?;

    Ok(HttpResponse::Ok().json(LoginResponse { 
        access_token,
        refresh_token,
        profile: user
    }))
}


/// Logout endpoint
#[utoipa::path(
    post,
    path = "/api/auth/logout"
)]
pub async fn logout() -> impl Responder {
    HttpResponse::Ok().json(MessageResponse { message: "Bye".to_string() })
}


/// Get user details for the current user
#[utoipa::path(
    post,
    path = "/api/auth/me",
    responses(
        (status = 200, response = User),
    ),
)]
pub async fn me(data: web::Data<AppState>, jwt: JwtMiddleware) -> Result<HttpResponse, ServiceError> {
    let user = auth_service::user_details(&data.db, jwt.user_id)?;
    Ok(HttpResponse::Ok().json(user))
}
