use actix_web::{get, web, HttpResponse};
use chrono::Utc;

use crate::{api::models::MessageResponse, errors::ServiceError, AppState};

/// Check the health of the SideStore ID service
#[utoipa::path(
    get,
    path = "/api/health",
    responses(
        (status = 200, description = "The service is up and running without any issues.", body = DateTime<Utc>)
    )
)]
#[get("/health")]
async fn ping(data: web::Data<AppState>) -> Result<HttpResponse, ServiceError> {
    data.db.get().map_err(|e| {
        log::error!("Database health check failed: {:?}", e);
        ServiceError::InternalServerError {
            error_message: "Database connection is down".to_string(),
        }
    })?;
    Ok(HttpResponse::Ok().json(MessageResponse {
        message: Utc::now().naive_utc().to_string(),
    }))
}
