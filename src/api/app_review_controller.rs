use std::path::Path;
use actix_web::{web, http::header::{ContentDisposition, DispositionType, DispositionParam}, HttpResponse};
use actix_files::NamedFile;
use log::debug;

use crate::{
    AppState,
    errors::{ServiceError, ErrorResponse},
    constants::REVIEWS_SIGNING_PUBLIC_KEY_NAME,
    util::review_signing::sign_review,
    middlewares::auth::JwtMiddleware,
    db::models::{app_review::AppReviewSignature, DbModel},
};
use crate::api::utils::enforce_scope;
use crate::auth::JwtTokenScope;

use super::models::app_reviews::{
    AppReviewSignatureRequest, AppReviewSignatureResponse,
    AppReviewSignatureData, AppReviewStatus,
    AppReviewDeletionRequest,
    UserAppReview, UserAppReviewList,
};


/// Get public signing key
/// 
/// Get public key to verify app review signatures, as an X.509 PEM certificate.
#[utoipa::path(
    get,
    path = "/api/reviews/public_key",
    responses(
        (
            status = 200, description = "Public key offered for download",
            content_type = "application/x-x509-ca-cert", body = String,
        )
    ),
)]
pub async fn get_public_key(data: web::Data<AppState>) -> Result<NamedFile, ServiceError> {
    let storage_path = Path::new(&data.env.storage_path);
    let file = NamedFile::open(storage_path.join(REVIEWS_SIGNING_PUBLIC_KEY_NAME))
        .map_err(|e| ServiceError::InternalServerError { error_message: e.to_string()})?
        .use_last_modified(true)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![
                DispositionParam::Filename("public_key.pem".to_string())
            ],
        });
    Ok(file)
}


/// Sign an app review
/// 
/// Save the app review metadata only and generate a signature for the app review that can be verified with the public key.
#[utoipa::path(
    post,
    path = "/api/reviews/sign",
    request_body = AppReviewSignatureRequest,
    responses(
        (status = 200, response = AppReviewSignatureResponse),
        (status = 401, description = "User authentication failed."),
        (status = 500, response = ErrorResponse),
    )
)]
pub async fn sign(body: web::Json<AppReviewSignatureRequest>, data: web::Data<AppState>, jwt: JwtMiddleware) -> Result<HttpResponse, ServiceError> {
    enforce_scope(&jwt, JwtTokenScope::Full)?;

    let mut review = match AppReviewSignature::find_by_user_id(
        &jwt.user_id, &body.source_identifier, &body.app_bundle_id, &mut data.db.get().unwrap()
    ) {
        Ok(mut review) => {
            debug!("User already has a review: {:?}. Update it.", review);
            review.review_rating = Some(body.review_rating.into());
            review.app_version = Some(body.version_number.to_string());
            review
                .update(&mut data.db.get().unwrap())
                .map_err(|e| {
                    debug!("Error updating review: {}", e);
                    ServiceError::InternalServerError { error_message: "Failed to update review".to_string() }
                })?
        },
        Err(_) => {
            debug!("User didn't leave a review for this app yet. Create one.");

            let mut review = AppReviewSignature::new(&body, &jwt.user_id);
            review.sequence_number = AppReviewSignature::find_latest_sequence_number(
                &body.source_identifier, &body.app_bundle_id, &mut data.db.get().unwrap()
            ).map_err(|e| {
                debug!("Error finding latest sequence number: {}", e);
                ServiceError::InternalServerError { error_message: "Failed to find latest sequence number".to_string() }
            })?;

            debug!("Latest sequence number: {}", review.sequence_number);
            review
                .insert(&mut data.db.get().unwrap())
                .map_err(|e| {
                    debug!("Error inserting review: {}", e);
                    ServiceError::InternalServerError { error_message: "Failed to save review".to_string() }
                })?
        }
    };

    let review_data = AppReviewSignatureData::from_signature_request(&body, &review);
    let signature = sign_review(&review_data, &data.review_signing_key)
        .map_err(|e| {
            debug!("Error signing review: {}", e);
            ServiceError::InternalServerError { error_message: "Failed to sign review".to_string() }
        })?;

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



/// Get all app reviews for the current user
#[utoipa::path(
    get,
    path = "/api/reviews",
    responses(
        (status = 200, response = UserAppReviewList),
        (status = 401, description = "User authentication failed."),
        (status = 500, response = ErrorResponse),
    )
)]
pub async fn get(data: web::Data<AppState>, jwt: JwtMiddleware) -> Result<HttpResponse, ServiceError> {
    enforce_scope(&jwt, JwtTokenScope::Full)?;

    let reviews: Vec<UserAppReview> = AppReviewSignature::find_all_by_user_id(&jwt.user_id, &mut data.db.get().unwrap())
        .map_err(|e| {
            log::debug!("Failed to get app reviews for user {:?}: {:?}", jwt.user_id, e);
            ServiceError::NotFound { error_message: "Couldn't find any reviews for the requesting user".to_string() }
        })?
        .iter()
        .map(|r| UserAppReview::from(r))
        .collect();

    Ok(HttpResponse::Ok().json(reviews))
}


/// Delete the app review for an app
#[utoipa::path(
    delete,
    path = "/api/reviews",
    request_body = AppReviewDeletionRequest,
    responses(
        (status = 200, response = UserAppReviewList),
        (status = 401, description = "User authentication failed."),
        (status = 500, response = ErrorResponse),
    )
)]
pub async fn delete(body: web::Json<AppReviewDeletionRequest>, data: web::Data<AppState>, jwt: JwtMiddleware) -> Result<HttpResponse, ServiceError> {
    enforce_scope(&jwt, JwtTokenScope::Full)?;

    let mut review = AppReviewSignature::find_by_user_id(
        &jwt.user_id, &body.source_identifier, &body.app_bundle_id, &mut data.db.get().unwrap()
    ).map_err(|_| ServiceError::NotFound { error_message: "You didn't review this app yet.".to_string() })?;

    debug!("Found the user's review: {:?}. Delete it.", review);
    review.status = AppReviewStatus::Deleted.into();
    review.review_rating = None;
    review.app_version = None;

    let mut review = review.update(&mut data.db.get().unwrap())
        .map_err(|e| {
            debug!("Error updating review: {}", e);
            ServiceError::InternalServerError { error_message: "Failed to update review".to_string() }
        })?;

    let review_data = AppReviewSignatureData::from_deletion_request(&body, &review);
    let signature = sign_review(&review_data, &data.review_signing_key)?;
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
