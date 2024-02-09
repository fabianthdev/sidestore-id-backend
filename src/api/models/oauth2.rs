use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum OAuth2AuthorizationResult {
    #[serde(rename = "allow")]
    Allow,

    #[serde(rename = "deny")]
    Deny,
}

impl From<&str> for OAuth2AuthorizationResult {
    fn from(value: &str) -> Self {
        match value {
            "allow" => Self::Allow,
            _ => Self::Deny,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OAuthRedirectResponse {
    pub redirect_url: String,
}
