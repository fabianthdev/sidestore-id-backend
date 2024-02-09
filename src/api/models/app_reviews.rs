use serde::{Deserialize, Serialize};
use utoipa::{ToResponse, ToSchema};

use crate::db::models::app_review::AppReviewSignature;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct AppReviewSignatureRequest {
    pub source_identifier: String,
    pub app_bundle_id: String,
    pub version_number: String,
    pub review_rating: u8,
    pub review_title: String,
    pub review_body: String,
}

#[derive(Serialize, Deserialize, ToResponse)]
pub struct AppReviewSignatureResponse {
    pub sequence_number: i32,
    pub review_date: i64,
    pub signature: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct AppReviewDeletionRequest {
    pub source_identifier: String,
    pub app_bundle_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToResponse, ToSchema)]
pub enum AppReviewStatus {
    #[serde(rename = "published")]
    Published,
    #[serde(rename = "deleted")]
    Deleted,
}

impl From<AppReviewStatus> for String {
    fn from(status: AppReviewStatus) -> Self {
        match status {
            AppReviewStatus::Published => "published".to_string(),
            AppReviewStatus::Deleted => "deleted".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AppReviewSignatureData {
    pub sidestore_user_id: String,
    pub status: AppReviewStatus,
    pub sequence_number: i32,
    pub source_identifier: String,
    pub app_bundle_identifier: String,
    pub version_number: Option<String>,
    pub review_rating: Option<u8>,
    pub review_title: Option<String>,
    pub review_body: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl AppReviewSignatureData {
    pub fn from_signature_request(
        request: &AppReviewSignatureRequest,
        review: &AppReviewSignature,
    ) -> AppReviewSignatureData {
        AppReviewSignatureData {
            sidestore_user_id: review.user_id.to_string(),
            status: AppReviewStatus::Published,
            sequence_number: review.sequence_number.clone(),
            source_identifier: review.source_id.clone(),
            app_bundle_identifier: review.app_bundle_id.clone(),
            version_number: Some(request.version_number.clone()),
            review_rating: Some(request.review_rating),
            review_title: Some(request.review_title.clone()),
            review_body: Some(request.review_body.clone()),
            created_at: review.created_at.timestamp(),
            updated_at: review.updated_at.timestamp(),
        }
    }

    pub fn from_deletion_request(
        request: &AppReviewDeletionRequest,
        review: &AppReviewSignature,
    ) -> AppReviewSignatureData {
        AppReviewSignatureData {
            sidestore_user_id: review.user_id.to_string(),
            status: AppReviewStatus::Deleted,
            sequence_number: review.sequence_number.clone(),
            source_identifier: review.source_id.clone(),
            app_bundle_identifier: request.app_bundle_id.clone(),
            version_number: None,
            review_rating: None,
            review_title: None,
            review_body: None,
            created_at: review.created_at.timestamp(),
            updated_at: review.updated_at.timestamp(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToResponse, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserAppReview {
    pub id: String,
    pub status: AppReviewStatus,
    pub source_identifier: String,
    pub app_bundle_identifier: String,
    pub version_number: Option<String>,
    pub review_rating: Option<i32>,
    pub date: i64,
    pub signature: Option<String>,
}

impl From<&AppReviewSignature> for UserAppReview {
    fn from(value: &AppReviewSignature) -> Self {
        UserAppReview {
            id: value.id.clone(),
            status: AppReviewStatus::Published, // TODO: Make this dynamic
            source_identifier: value.source_id.clone(),
            app_bundle_identifier: value.app_bundle_id.clone(),
            version_number: value.app_version.clone(),
            review_rating: value.review_rating,
            date: value.updated_at.timestamp(),
            signature: value.signature.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToResponse)]
#[serde(rename_all = "camelCase")]
pub struct UserAppReviewList(Vec<UserAppReview>);
