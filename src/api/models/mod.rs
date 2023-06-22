pub mod auth;
pub mod app_reviews;

use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize)]
pub struct MessageResponse {
    pub message: String,
}
