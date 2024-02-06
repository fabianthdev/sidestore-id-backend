pub mod auth;
pub mod app_reviews;
pub mod oauth2;

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct MessageResponse {
    pub message: String,
}
