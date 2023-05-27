use actix_web::{web, HttpResponse, Responder};

use crate::AppState;
use crate::middlewares::auth::JwtMiddleware;
use crate::services::auth_service;
use crate::{db::models::user::UserDTO, errors::ServiceError};

use super::models::MessageResponse;
use super::models::auth::{LoginRequest, LoginResponse, SignupRequest, SignupResponse};


pub async fn signup(body: web::Json<SignupRequest>, data: web::Data<AppState>) -> Result<HttpResponse, ServiceError> {
    let user_dto = UserDTO { email: body.email.clone(), password: body.password.clone(), username: None };
    match auth_service::signup(user_dto, &data.db, &data.env) {
        Ok((user, access_token, refresh_token)) => Ok(HttpResponse::Ok().json(SignupResponse { 
            access_token,
            refresh_token,
            profile: user
        })),
        Err(err) => Err(err)
    }
}

pub async fn login(body: web::Json<LoginRequest>, data: web::Data<AppState>) -> Result<HttpResponse, ServiceError> {
    let user_dto = UserDTO { email: body.email.clone(), password: body.password.clone(), username: None };
    match auth_service::login(user_dto, &data.db, &data.env) {
        Ok((user, access_token, refresh_token)) => Ok(HttpResponse::Ok().json(LoginResponse { 
            access_token,
            refresh_token,
            profile: user
         })),
        Err(err) => Err(err)
    }
}

pub async fn refresh(data: web::Data<AppState>, jwt: JwtMiddleware) -> Result<HttpResponse, ServiceError> {
    match auth_service::refresh(&data.db, &data.env, jwt.user_id) {
        Ok((user, access_token, refresh_token)) => Ok(HttpResponse::Ok().json(LoginResponse { 
            access_token,
            refresh_token,
            profile: user
         })),
        Err(err) => Err(err)
    }
}

pub async fn logout() -> impl Responder {
    HttpResponse::Ok().json(MessageResponse { message: "Bye".to_string() })
}

pub async fn me(data: web::Data<AppState>, jwt: JwtMiddleware) -> Result<HttpResponse, ServiceError> {
    match auth_service::user_details(&data.db, jwt.user_id) {
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
        Err(err) => Err(err)
    }
}
