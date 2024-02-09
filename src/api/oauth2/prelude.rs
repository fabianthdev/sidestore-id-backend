use crate::auth::JwtTokenScope;
use oxide_auth::primitives::scope::{ParseScopeErr, Scope};

impl JwtTokenScope {
    pub fn into_oauth_scope(self) -> Result<Scope, ParseScopeErr> {
        let scope_name = match self {
            JwtTokenScope::Full => "full",
            JwtTokenScope::Profile => "profile",
        };
        scope_name.parse()
    }
}
