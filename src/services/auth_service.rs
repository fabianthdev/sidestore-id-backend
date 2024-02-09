use crate::auth::{create_auth_tokens, JwtTokenScope};
use crate::config::Config;
use crate::db::models::user::{User, UserDTO};
use crate::db::Pool;
use crate::errors::ServiceError;

pub type UserAndTokens = (User, String, String);

pub fn signup(
    user_dto: UserDTO,
    pool: &Pool,
    config: &Config,
) -> Result<UserAndTokens, ServiceError> {
    let conn = &mut pool.get().unwrap();
    if User::find_by_email(&user_dto.email, conn).is_ok() {
        return Err(ServiceError::BadRequest {
            error_message: "Email already exists".to_string(),
        });
    }

    let password_hash = bcrypt::hash(user_dto.password, bcrypt::DEFAULT_COST).unwrap();
    let user = User::new(&user_dto.email, &password_hash);

    let user = match user.clone().insert(conn) {
        Ok(user) => user,
        Err(_) => {
            return Err(ServiceError::InternalServerError {
                error_message: "User could not be saved".to_string(),
            })
        }
    };

    let (access_token, refresh_token) = match create_auth_tokens(&user, config, JwtTokenScope::Full)
    {
        Ok(tokens) => tokens,
        Err(e) => return Err(ServiceError::InternalServerError { error_message: e }),
    };

    return Ok((user, access_token, refresh_token));
}

pub fn login(
    user_dto: UserDTO,
    pool: &Pool,
    config: &Config,
) -> Result<UserAndTokens, ServiceError> {
    let conn = &mut pool.get().unwrap();
    let user = match User::find_by_email(&user_dto.email, conn) {
        Ok(user) => user,
        Err(_) => {
            return Err(ServiceError::Unauthorized {
                error_message: "User not found".to_string(),
            })
        }
    };

    let password_matches = bcrypt::verify(&user_dto.password, &user.password_hash);
    if password_matches.is_err() {
        return Err(ServiceError::InternalServerError {
            error_message: "Error verifying password".to_string(),
        });
    } else if password_matches.is_ok() && !password_matches.unwrap() {
        return Err(ServiceError::Unauthorized {
            error_message: "Password is incorrect".to_string(),
        });
    }

    let (access_token, refresh_token) = match create_auth_tokens(&user, config, JwtTokenScope::Full)
    {
        Ok(tokens) => tokens,
        Err(e) => return Err(ServiceError::InternalServerError { error_message: e }),
    };

    Ok((user, access_token, refresh_token))
}

pub fn refresh(
    pool: &Pool,
    config: &Config,
    user_id: uuid::Uuid,
) -> Result<UserAndTokens, ServiceError> {
    let conn = &mut pool.get().unwrap();

    let user = match User::find_by_id(&user_id, conn) {
        Ok(user) => user,
        Err(_) => {
            return Err(ServiceError::Unauthorized {
                error_message: "User not found".to_string(),
            })
        }
    };

    let (access_token, refresh_token) = match create_auth_tokens(&user, config, JwtTokenScope::Full)
    {
        Ok(tokens) => tokens,
        Err(e) => return Err(ServiceError::InternalServerError { error_message: e }),
    };

    Ok((user, access_token, refresh_token))
}

pub fn user_details(pool: &Pool, user_id: uuid::Uuid) -> Result<User, ServiceError> {
    let conn = &mut pool.get().unwrap();

    match User::find_by_id(&user_id, conn) {
        Ok(user) => Ok(user),
        Err(_) => Err(ServiceError::Unauthorized {
            error_message: "User not found".to_string(),
        }),
    }
}
