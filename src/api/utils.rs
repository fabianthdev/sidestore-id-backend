use crate::auth::JwtTokenScope;
use crate::errors::ServiceError;
use crate::middlewares::auth::JwtMiddleware;

pub fn enforce_scope(jwt: &JwtMiddleware, scope: JwtTokenScope) -> Result<(), ServiceError> {
    if jwt.scope > scope {
        Err(ServiceError::Unauthorized {
            error_message: "Invalid token scope.".to_string(),
        })
    } else {
        Ok(())
    }
}
