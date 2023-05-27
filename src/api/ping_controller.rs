use actix_web::{get, HttpResponse};
use chrono::Utc;

use crate::api::models::MessageResponse;

#[get("/health")]
async fn ping() -> HttpResponse {
    HttpResponse::Ok().json(MessageResponse { message: Utc::now().naive_utc().to_string() })
}
