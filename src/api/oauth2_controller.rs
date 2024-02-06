use std::ops::Deref;

use actix::Addr;
use actix_web::web;
use oxide_auth::endpoint::{QueryParameter, WebResponse};

use crate::{api::oauth2::state::OAuth2State, AppState, middlewares::auth::JwtMiddleware};
use crate::api::models::oauth2::{OAuth2AuthorizationResult, OAuthRedirectResponse};
use crate::api::utils::enforce_scope;
use crate::auth::JwtTokenScope;
use crate::services::auth_service;

use super::oauth2::oxide_auth_actix::{Authorize, ClientCredentials, OAuthOperation, OAuthRequest, OAuthResponse, Refresh, Token, WebError};
use super::oauth2::state::Extras;

pub async fn get_authorize(req: OAuthRequest, state: web::Data<Addr<OAuth2State>>) -> Result<OAuthResponse, WebError> {
    state.send(Authorize(req)
        .wrap(Extras::AuthGet))
        .await?
}

pub async fn post_authorize(req: OAuthRequest, state: web::Data<Addr<OAuth2State>>, data: web::Data<AppState>, jwt: JwtMiddleware) -> Result<OAuthResponse, WebError> {
    enforce_scope(&jwt, JwtTokenScope::Full).map_err(|e| WebError::Authorization)?;

    let user = auth_service::user_details(&data.db, jwt.user_id).map_err(|e| WebError::Authorization)?;

    let result_string = req.query()
        .and_then(|q| q.unique_value("result"))
        .ok_or(WebError::Query)?;

    let result = match result_string.deref() {
        "allow" => Ok(OAuth2AuthorizationResult::Allow),
        "deny" => Ok(OAuth2AuthorizationResult::Deny),
        _ => Err(WebError::Query),
    }?;

    let response = state.send(Authorize(req)
        .wrap(Extras::AuthPost(user, result)))
        .await?;

    match response {
        Ok(r) => {
            let headers = r.get_headers();
            let location = headers.get("location");
            if let Some(redirect_url) = location {
                let body = serde_json::to_string(&OAuthRedirectResponse {
                    redirect_url: redirect_url.to_str().unwrap().to_string()
                }).unwrap();
                let mut modified_response = OAuthResponse::ok();
                _ = modified_response.body_json(&body);
                Ok(modified_response)
            } else {
                Ok(r)
            }
        },
        Err(e) => Err(e)
    }
}

pub async fn token(req: OAuthRequest, state: web::Data<Addr<OAuth2State>>) -> Result<OAuthResponse, WebError> {
    let grant_type = req.body().and_then(|body| body.unique_value("grant_type"));

    match grant_type.as_deref() {
        Some("client_credentials") => {
            state
                .send(ClientCredentials(req).wrap(Extras::ClientCredentials))
                .await?
        },
        // Each flow will validate the grant_type again, so we can let one case handle
        // any incorrect or unsupported options.
        _ => state.send(Token(req).wrap(Extras::Nothing)).await?,
    }
}

async fn refresh(
    (req, state): (OAuthRequest, web::Data<Addr<OAuth2State>>),
) -> Result<OAuthResponse, WebError> {
    state.send(Refresh(req).wrap(Extras::Nothing)).await?
}
