
pub mod review_signing {
    use base64::{Engine as _, engine::general_purpose::STANDARD as base64_engine};
    use ed25519_dalek::{Signer, SigningKey};
    use ed25519_dalek::pkcs8::spki::der::pem::LineEnding;
    use ed25519_dalek::pkcs8::{EncodePrivateKey, EncodePublicKey, DecodePrivateKey};
    use log::{info, debug};
    use serde_json;

    use crate::{config::Config, constants, errors::ServiceError, api::models::app_reviews::AppReviewSignatureData};

    pub fn create_or_load_review_signing_key(config: &Config) -> Result<SigningKey, String> {
        let storage_path = std::path::Path::new(&config.storage_path);
        let private_key_path = storage_path.join(constants::REVIEWS_SIGNING_PRIVATE_KEY_NAME);
        let public_key_path = storage_path.join(constants::REVIEWS_SIGNING_PUBLIC_KEY_NAME);
    
        use rand::rngs::OsRng;
    
        let signing_key: SigningKey;
        if !private_key_path.exists() || !public_key_path.exists() {
            info!("Generating new review signing keypair...");
    
            let mut csprng = OsRng{};
            signing_key = SigningKey::generate(&mut csprng);
    
            match signing_key.verifying_key().to_public_key_pem(LineEnding::LF) {
                Ok(public_key_pem) => {
                    match std::fs::write(&public_key_path, public_key_pem) {
                        Ok(_) => info!("Public key written to {}", public_key_path.display()),
                        Err(e) => return Err(format!("Failed to write public key to file: {}", e))
                    }
                },
                Err(e) => return Err(format!("Failed to encode public key: {}", e))
            }
    
            match signing_key.to_pkcs8_pem(LineEnding::LF) {
                Ok(private_key_pem) => {
                    match std::fs::write(&private_key_path, private_key_pem) {
                        Ok(_) => info!("Private key written to {}", private_key_path.display()),
                        Err(e) => return Err(format!("Failed to write private key to file: {}", e))
                    }
                },
                Err(e) => return Err(format!("Failed to encode private key: {}", e))
            }
            info!("New review signing keypair generated and written to disk")
        } else {
            info!("Loading existing review signing keypair...");
    
            let private_key_pem = match std::fs::read_to_string(&private_key_path) {
                Ok(private_key_pem) => private_key_pem,
                Err(e) => return Err(format!("Failed to read private key from file: {}", e))
            };
    
            signing_key = match SigningKey::from_pkcs8_pem(&private_key_pem) {
                Ok(signing_key) => signing_key,
                Err(e) => return Err(format!("Failed to decode private key: {}", e))
            };
    
            info!("Existing review signing keypair loaded from disk")
        }
    
        Ok(signing_key)
    }
    

    pub fn sign_review(review_data: AppReviewSignatureData, signing_key: &SigningKey) -> Result<String, ServiceError> {
        let review_data_json = match serde_json::to_string(&review_data) {
            Ok(json) => json,
            Err(e) => {
                debug!("Error serializing review data: {}", e);
                return Err(ServiceError::InternalServerError { error_message: "Failed to serialize review data".to_string() })
            }
        };
        debug!("Review data: {}", review_data_json);
        let signature = signing_key.sign(review_data_json.as_bytes());
        Ok(base64_engine.encode(&signature.to_bytes()))
    }

    #[cfg(test)]
    mod tests {
        use crate::api::models::app_reviews::AppReviewStatus;
        use super::*;

        #[test]
        fn test_create_or_loead_review_signing_key() {
            
        }

        #[test]
        fn test_sign_review() {
    
            let config = Config {
                host: "localhost".to_string(),
                port: 8080,
                jwt_secret: "secret".to_string(),
                jwt_issuer: "io.sidestore.SideStore-ID".to_string(),
                jwt_expiration: 3600,
                jwt_refresh_expiration: 86400,
                cors_origin: "*".to_string(),
                database_url: "sqlite://test.db".to_string(),
                storage_path: "./test-storage".to_string(),
            };
            let signing_key = create_or_load_review_signing_key(&config).unwrap();
            let review_data = AppReviewSignatureData {
                sidestore_user_id: "uuid-1234-5678-9012-3456".to_string(),
                status: AppReviewStatus::Published.into(),
                sequence_number: 69,
                source_identifier: "io.sidestore.Connect".to_string(),
                app_bundle_identifier: "com.SideStore.SideStore".to_string(),
                version_number: Some("4.2.0".to_string()),
                review_rating: Some(5),
                review_title: Some("This is a test review".to_string()),
                review_body: Some("This is a test review body".to_string()),
                created_at: 1682007600,
                updated_at: 1682007600
            };
            
            let review_data_json = serde_json::to_string(&review_data).unwrap();
            assert_eq!(review_data_json, "{\"sidestore_user_id\":\"uuid-1234-5678-9012-3456\",\"status\":\"published\",\"sequence_number\":69,\"source_identifier\":\"io.sidestore.Connect\",\"app_bundle_identifier\":\"com.SideStore.SideStore\",\"version_number\":\"4.2.0\",\"review_rating\":5,\"review_title\":\"This is a test review\",\"review_body\":\"This is a test review body\",\"created_at\":1682007600,\"updated_at\":1682007600}" );
    
            let signature = sign_review(review_data, &signing_key).unwrap();
            println!("Signature: {}", signature);
            assert_eq!(signature, "W+XULDRHnhOFxWi5NJS0sMr11+9128XqZADIFp3NPmNky6sgIZ5MCR8NPn0Ee64W7KFhozlydendO1LAE8SWDA==".to_string())
        }
    }
}