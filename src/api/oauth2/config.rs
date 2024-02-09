use oxide_auth::primitives::prelude::Client;
use oxide_auth::primitives::registrar::{ExactUrl, RegisteredUrl};
use serde::Deserialize;

use crate::auth::JwtTokenScope;

#[derive(Debug, Deserialize)]
pub struct OAuthConfig {
    pub clients: Vec<OAuthClient>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OAuthClient {
    client_id: String,
    client_secret: String,
    scope: JwtTokenScope,
    redirect_uri: String,
    additional_redirect_uris: Option<Vec<String>>,
}

impl Into<Client> for &OAuthClient {
    fn into(self) -> Client {
        let mut client = Client::confidential(
            &self.client_id,
            RegisteredUrl::Exact(ExactUrl::new(self.redirect_uri.to_string()).unwrap()),
            self.scope.clone().into_oauth_scope().unwrap(),
            self.client_secret.as_bytes(),
        );

        if let Some(additional_redirect_urls) = self.additional_redirect_uris.clone() {
            client = client.with_additional_redirect_uris(
                additional_redirect_urls
                    .iter()
                    .map(|url| RegisteredUrl::Exact(ExactUrl::new(url.to_string()).unwrap()))
                    .collect(),
            )
        }

        client
    }
}
