use std::path::Path;
use actix_web::{web, http::header::{ContentDisposition, DispositionType, DispositionParam}, HttpResponse};
use actix_files::NamedFile;
use log::debug;

use crate::{
    AppState,
    errors::ServiceError,
    constants::REVIEWS_SIGNING_PUBLIC_KEY_NAME,
    util::review_signing::sign_review,
    middlewares::auth::JwtMiddleware,
    db::models::{app_review::AppReviewSignature, DbModel},
};

use super::models::app_reviews::{
    AppReviewSignatureRequest, AppReviewSignatureResponse,
    AppReviewSignatureData, AppReviewStatus,
    AppReviewDeletionRequest
};


pub async fn get_public_key(data: web::Data<AppState>) -> Result<NamedFile, ServiceError> {
    let storage_path = Path::new(&data.env.storage_path);
    match NamedFile::open(storage_path.join(REVIEWS_SIGNING_PUBLIC_KEY_NAME)) {
        Ok(file) => Ok(file
            .use_last_modified(true)
            .set_content_disposition(ContentDisposition {
                disposition: DispositionType::Attachment,
                parameters: vec![
                    DispositionParam::Filename("public_key.pem".to_string())
                ],
            })
        ),
        Err(e) => Err(ServiceError::InternalServerError { error_message: e.to_string()})
    }
}

pub async fn sign(body: web::Json<AppReviewSignatureRequest>, data: web::Data<AppState>, jwt: JwtMiddleware) -> Result<HttpResponse, ServiceError> {
    let mut review = match AppReviewSignature::find_by_user_id(
        &jwt.user_id, &body.source_identifier, &body.app_bundle_id, &mut data.db.get().unwrap()
    ) {
        Ok(mut review) => {
            debug!("User already has a review: {:?}. Update it.", review);
            review.review_rating = Some(body.review_rating.into());
            review.app_version = Some(body.version_number.to_string());
            match review.update(&mut data.db.get().unwrap()) {
                Ok(review) => review,
                Err(e) => {
                    debug!("Error updating review: {}", e);
                    return Err(ServiceError::InternalServerError { error_message: "Failed to update review".to_string() })
                }
            }
        },
        Err(_) => {
            debug!("User didn't leave a review for this app yet. Create one.");
            let mut review = AppReviewSignature::new(&body, &jwt.user_id);
            review.sequence_number = match AppReviewSignature::find_latest_sequence_number(
                &body.source_identifier, &body.app_bundle_id, &mut data.db.get().unwrap()
            ) {
                Ok(sequence_number) => sequence_number + 1,
                Err(e) => {
                    debug!("Error finding latest sequence number: {}", e);
                    return Err(ServiceError::InternalServerError { error_message: "Failed to find latest sequence number".to_string() })
                }
            };
            debug!("Latest sequence number: {}", review.sequence_number);
            match review.insert(&mut data.db.get().unwrap()) {
                Ok(review) => review,
                Err(e) => {
                    debug!("Error inserting review: {}", e);
                    return Err(ServiceError::InternalServerError { error_message: "Failed to insert review".to_string() })
                }
            }
        }
    };

    let review_data = AppReviewSignatureData::from_signature_request(&body, &review);
    let signature = match sign_review(&review_data, &data.review_signing_key) {
        Ok(signature) => signature,
        Err(e) => {
            debug!("Error signing review: {}", e);
            return Err(ServiceError::InternalServerError { error_message: "Failed to sign review".to_string() })
        }
    };

    review.signature = Some(signature.clone().to_string());
    match review.update(&mut data.db.get().unwrap()) {
        Ok(_) => Ok(HttpResponse::Ok().json(AppReviewSignatureResponse {
            sequence_number: review.sequence_number,
            review_date: review_data.updated_at,
            signature
        })),
        Err(e) => {
            debug!("Error updating review: {}", e);
            return Err(ServiceError::InternalServerError { error_message: "Failed to update review".to_string() })
        }
    }
}

pub async fn delete(body: web::Json<AppReviewDeletionRequest>, data: web::Data<AppState>, jwt: JwtMiddleware) -> Result<HttpResponse, ServiceError> {
    match AppReviewSignature::find_by_user_id(
        &jwt.user_id, &body.source_identifier, &body.app_bundle_id, &mut data.db.get().unwrap()
    ) {
        Ok(mut review) => {
            debug!("Found the user's review: {:?}. Delete it.", review);
            review.status = AppReviewStatus::Deleted.into();
            review.review_rating = None;
            review.app_version = None;
            let mut review = match review.update(&mut data.db.get().unwrap()) {
                Ok(review) => review,
                Err(e) => {
                    debug!("Error updating review: {}", e);
                    return Err(ServiceError::InternalServerError { error_message: "Failed to update review".to_string() })
                }
            };

            let review_data = AppReviewSignatureData::from_deletion_request(&body, &review);
            let signature = match sign_review(&review_data, &data.review_signing_key) {
                Ok(signature) => signature,
                Err(e) => {
                    debug!("Error signing review: {}", e);
                    return Err(ServiceError::InternalServerError { error_message: "Failed to sign review".to_string() })
                }
            };
            review.signature = Some(signature.clone().to_string());
            match review.update(&mut data.db.get().unwrap()) {
                Ok(_) => Ok(HttpResponse::Ok().json(AppReviewSignatureResponse {
                    sequence_number: review.sequence_number,
                    review_date: review_data.updated_at,
                    signature
                })),
                Err(e) => {
                    debug!("Error updating review: {}", e);
                    return Err(ServiceError::InternalServerError { error_message: "Failed to update review".to_string() })
                }
            }
        },
        Err(_) => Err(ServiceError::NotFound { error_message: "You didn't review this app yet.".to_string() })
    }
}
