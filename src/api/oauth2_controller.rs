use std::ops::Deref;

use actix::Addr;
use actix_web::web;
use oxide_auth::endpoint::{QueryParameter, WebResponse};

use crate::api::models::oauth2::{OAuth2AuthorizationResult, OAuthRedirectResponse};
use crate::api::utils::enforce_scope;
use crate::auth::JwtTokenScope;
use crate::services::auth_service;
use crate::{api::oauth2::state::OAuth2State, middlewares::auth::JwtMiddleware, AppState};

use super::oauth2::oxide_auth_actix::{
    Authorize, ClientCredentials, OAuthOperation, OAuthRequest, OAuthResponse, Refresh, Token,
    WebError,
};
use super::oauth2::state::Extras;

pub async fn get_authorize(
    req: OAuthRequest,
    state: web::Data<Addr<OAuth2State>>,
    data: web::Data<AppState>,
    jwt: JwtMiddleware,
) -> Result<OAuthResponse, WebError> {
    // Check if the user has authorized this client before
    if let Some(user) = auth_service::user_details(&data.db, jwt.user_id).ok() {
        let client_id = req
            .query()
            .and_then(|params| params.unique_value("client_id"))
            .unwrap_or_default();

        let conn = &mut data.db.get().unwrap();
        if user.has_authorized_oauth_client(&client_id, conn) {
            return state
                .send(
                    Authorize(req)
                        .wrap(Extras::AuthPost(user.id, OAuth2AuthorizationResult::Allow)),
                )
                .await?;
        }
    }

    state.send(Authorize(req).wrap(Extras::AuthGet)).await?
}

pub async fn post_authorize(
    req: OAuthRequest,
    state: web::Data<Addr<OAuth2State>>,
    data: web::Data<AppState>,
    jwt: JwtMiddleware,
) -> Result<OAuthResponse, WebError> {
    enforce_scope(&jwt, JwtTokenScope::Full).map_err(|_| WebError::Authorization)?;

    let mut user =
        auth_service::user_details(&data.db, jwt.user_id).map_err(|_| WebError::Authorization)?;

    let result_string = req
        .query()
        .and_then(|q| q.unique_value("result"))
        .ok_or(WebError::Query)?;

    let result = OAuth2AuthorizationResult::from(result_string.deref());
    let response = state
        .send(Authorize(req.clone()).wrap(Extras::AuthPost(user.id.to_string(), result.clone())))
        .await?;

    match response {
        Ok(r) => {
            // Save the authorization
            let client_id = req
                .query()
                .and_then(|params| params.unique_value("client_id"))
                .unwrap_or_default();
            let conn = &mut data.db.get().unwrap();

            match result {
                OAuth2AuthorizationResult::Allow => {
                    user.save_oauth_client_authorization(&client_id, conn)
                        .map_err(|_| WebError::Authorization)?;
                }
                OAuth2AuthorizationResult::Deny => {
                    user.remove_oauth_client_authorization(&client_id, conn)
                        .map_err(|_| WebError::Authorization)?;
                }
            };

            let headers = r.get_headers();
            let location = headers.get("location");
            if let Some(redirect_url) = location {
                let body = serde_json::to_string(&OAuthRedirectResponse {
                    redirect_url: redirect_url.to_str().unwrap().to_string(),
                })
                .unwrap();
                let mut modified_response = OAuthResponse::ok();
                _ = modified_response.body_json(&body);
                Ok(modified_response)
            } else {
                Ok(r)
            }
        }
        Err(e) => Err(e),
    }
}

pub async fn token(
    req: OAuthRequest,
    state: web::Data<Addr<OAuth2State>>,
) -> Result<OAuthResponse, WebError> {
    let grant_type = req.body().and_then(|body| body.unique_value("grant_type"));

    match grant_type.as_deref() {
        Some("client_credentials") => {
            state
                .send(ClientCredentials(req).wrap(Extras::ClientCredentials))
                .await?
        }
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
