use std::future::{ready, Ready};

use actix_web::error::ErrorUnauthorized;
use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{http, web, FromRequest, HttpMessage, HttpRequest};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::auth::{JwtToken, JwtTokenScope, JwtTokenType};
use crate::constants::{OAUTH_GET_API_PATH, REFRESH_API_PATH, UNPROTECTED_API_PATHS};
use crate::AppState;

pub struct JwtMiddleware {
    pub user_id: uuid::Uuid,
    pub scope: JwtTokenScope,
}

impl FromRequest for JwtMiddleware {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        if UNPROTECTED_API_PATHS.contains(&req.path()) {
            return ready(Ok(JwtMiddleware {
                user_id: uuid::Uuid::nil(),
                scope: JwtTokenScope::Full,
            }));
        }

        let expected_token_type = match req.path() {
            REFRESH_API_PATH => JwtTokenType::Refresh,
            _ => JwtTokenType::Access,
        };
        let data = req.app_data::<web::Data<AppState>>().unwrap();

        // Get token from Authorization header
        let token = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .ok_or(ErrorUnauthorized("Authorization header not found"))
            .and_then(|header_value| {
                header_value
                    .to_str()
                    .map_err(|_| ErrorUnauthorized("Invalid authorization header"))
            })
            .and_then(|header_value| {
                header_value
                    .replace("Bearer ", "")
                    .parse::<String>()
                    .map_err(|_| ErrorUnauthorized("Invalid authorization header"))
            })
            .or(
                // Fallback to get token from cookie
                match expected_token_type {
                    JwtTokenType::Access => req.cookie("access_token"),
                    JwtTokenType::Refresh => req.cookie("refresh_token"),
                }
                .map(|cookie| cookie.value().to_string())
                .ok_or(ErrorUnauthorized("Authentication cookie not found")),
            );

        let token_str = match token {
            Ok(t) => t,
            Err(e) => {
                if req.path() == OAUTH_GET_API_PATH {
                    return ready(Ok(JwtMiddleware {
                        user_id: uuid::Uuid::nil(),
                        scope: JwtTokenScope::Profile,
                    }));
                }
                return ready(Err(e));
            }
        };

        let token = match decode::<JwtToken>(
            &token_str,
            &DecodingKey::from_secret(data.env.jwt_secret.as_ref()),
            &Validation::default(),
        ) {
            Ok(c) => c,
            Err(_) => return ready(Err(ErrorUnauthorized("Invalid token"))),
        };

        if token.claims.type_ != expected_token_type {
            return ready(Err(ErrorUnauthorized("Invalid token")));
        }

        if token.claims.exp < chrono::Utc::now().timestamp() {
            return ready(Err(ErrorUnauthorized("Token expired")));
        } else if token.claims.iat > chrono::Utc::now().timestamp() {
            return ready(Err(ErrorUnauthorized("Token used before issued")));
        }

        let user_id = uuid::Uuid::parse_str(token.claims.sub.as_str()).unwrap();
        req.extensions_mut()
            .insert::<uuid::Uuid>(user_id.to_owned());

        ready(Ok(JwtMiddleware {
            user_id,
            scope: token.claims.scope,
        }))
    }
}
