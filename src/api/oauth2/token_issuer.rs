use std::ops::Add;
use chrono::Utc;
use oxide_auth::endpoint::Issuer;
use oxide_auth::primitives::grant::Grant;
use oxide_auth::primitives::issuer::{IssuedToken, RefreshedToken, TokenType};
use crate::auth::{create_jwt_token, JwtTokenScope, JwtTokenType};
use crate::config::Config;

pub struct JwtTokenIssuer {
    config: Config,
}

impl JwtTokenIssuer {
    pub fn new(config: Config) -> JwtTokenIssuer {
        JwtTokenIssuer { config }
    }
}

impl Issuer for JwtTokenIssuer {
    fn issue(&mut self, grant: Grant) -> Result<IssuedToken, ()> {
        let token = create_jwt_token(&grant.owner_id, JwtTokenType::Access, &JwtTokenScope::Profile, &self.config)
            .map_err(|e| {
                log::error!("Failed to create oauth access token for user {}: {:?}", &grant.owner_id, e)
            })?;
        let refresh = create_jwt_token(&grant.owner_id, JwtTokenType::Refresh, &JwtTokenScope::Profile, &self.config)
            .map_err(|e|
                log::error!("Failed to create oauth access token for user {}: {:?}", &grant.owner_id, e)
            )?;

        Ok(IssuedToken {
            token,
            refresh: Some(refresh),
            until: Utc::now().add(chrono::Duration::seconds(self.config.jwt_expiration)),
            token_type: TokenType::Bearer,
        })
    }

    fn refresh(&mut self, _refresh: &str, _grant: Grant) -> Result<RefreshedToken, ()> {
        Err(())
    }

    fn recover_token<'a>(&'a self, _: &'a str) -> Result<Option<Grant>, ()> {
        Err(())
    }

    fn recover_refresh<'a>(&'a self, _: &'a str) -> Result<Option<Grant>, ()> {
        Err(())
    }
}
