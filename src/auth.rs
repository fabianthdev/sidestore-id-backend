use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::db::models::user::User;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub enum JwtTokenType {
    #[serde(rename = "access")]
    Access,
    #[serde(rename = "refresh")]
    Refresh,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub enum JwtTokenScope {
    #[serde(rename = "full")]
    Full = 0,

    #[serde(rename = "profile")]
    Profile = 1,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtToken {
    #[serde(rename = "type")]
    pub type_: JwtTokenType,
    pub iss: String,
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub fresh: bool,
    pub scope: JwtTokenScope,
}

pub fn create_auth_tokens(
    user: &User,
    config: &Config,
    scope: JwtTokenScope,
) -> Result<(String, String), String> {
    let access_token =
        match create_jwt_token(&user.id.to_string(), JwtTokenType::Access, &scope, config) {
            Ok(t) => t,
            Err(_) => return Err("Error generating access token".to_string()),
        };
    let refresh_token =
        match create_jwt_token(&user.id.to_string(), JwtTokenType::Refresh, &scope, config) {
            Ok(t) => t,
            Err(_) => return Err("Error generating refresh token".to_string()),
        };

    Ok((access_token, refresh_token))
}

pub fn create_jwt_token(
    user_id: &str,
    type_: JwtTokenType,
    scope: &JwtTokenScope,
    config: &Config,
) -> Result<String, String> {
    let expiration_seconds = match type_ {
        JwtTokenType::Access => config.jwt_expiration,
        JwtTokenType::Refresh => config.jwt_refresh_expiration,
    };
    let now = Utc::now();
    let iat = now.timestamp() as i64;
    let exp = (now + chrono::Duration::seconds(expiration_seconds)).timestamp() as i64;
    let token: JwtToken = JwtToken {
        type_,
        iss: config.jwt_issuer.clone(),
        sub: user_id.to_string(),
        iat,
        exp,
        fresh: false,
        scope: scope.clone(),
    };

    match encode(
        &Header::default(),
        &token,
        &EncodingKey::from_secret(config.jwt_secret.as_ref()),
    ) {
        Ok(t) => Ok(t),
        Err(_) => return Err("Error generating jwt token".to_string()),
    }
}
