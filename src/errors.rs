use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::{Display, Error};
use serde::Serialize;
use utoipa::ToResponse;

#[derive(Debug, Display, Error)]
pub enum ServiceError {
    #[display(fmt = "Internal Server Error")]
    InternalServerError { error_message: String },

    #[display(fmt = "{error_message}")]
    BadRequest { error_message: String },

    #[display(fmt = "Unauthorized")]
    Unauthorized { error_message: String },

    #[display(fmt = "{error_message}")]
    NotFound { error_message: String },

    #[display(fmt = "Validation error on field: {}", field)]
    ValidationError { field: String },
}

#[derive(Serialize, ToResponse)]
#[response(description = "The server encountered an error while trying to fulfill the request.")]
pub struct ErrorResponse {
    pub error_message: String,
}

impl error::ResponseError for ServiceError {
    fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::InternalServerError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ServiceError::BadRequest { .. } => StatusCode::BAD_REQUEST,
            ServiceError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            ServiceError::NotFound { .. } => StatusCode::NOT_FOUND,
            ServiceError::ValidationError { .. } => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .json(ErrorResponse {
                error_message: self.to_string(),
            })
    }
}
