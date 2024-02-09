use chrono::{NaiveDateTime, Utc};
use diesel::result::Error;
use diesel::sql_types::Uuid;
use diesel::{AsChangeset, ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl};
use log::error;
use serde::{Deserialize, Serialize};

use crate::api::models::app_reviews::{AppReviewSignatureRequest, AppReviewStatus};
use crate::db::schema::app_review_signatures;
use crate::db::Connection;

use super::{db_model, DbModel};

#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = app_review_signatures)]
pub struct AppReviewSignature {
    #[serde(default, skip_serializing)]
    pub id: String,
    pub user_id: String,
    pub status: String,
    pub sequence_number: i32,
    pub source_id: String,
    pub app_bundle_id: String,
    pub app_version: Option<String>,
    pub review_rating: Option<i32>,
    pub signature: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

db_model!(
    app_review_signatures::dsl::app_review_signatures,
    app_review_signatures::dsl::id,
    AppReviewSignature
);

impl AppReviewSignature {
    pub fn find_by_id(id: &uuid::Uuid, conn: &mut Connection) -> Result<Self, Error> {
        app_review_signatures::dsl::app_review_signatures
            .find(id.to_string())
            .get_result::<Self>(conn)
    }

    pub fn find_by_user_id(
        user_id: &uuid::Uuid,
        source_id: &str,
        app_bundle_id: &str,
        conn: &mut Connection,
    ) -> Result<Self, Error> {
        app_review_signatures::dsl::app_review_signatures
            .filter(app_review_signatures::user_id.eq(user_id.to_string()))
            .filter(app_review_signatures::source_id.eq(source_id.to_string()))
            .filter(app_review_signatures::app_bundle_id.eq(app_bundle_id.to_string()))
            .get_result::<Self>(conn)
    }

    pub fn find_all_by_user_id(
        user_id: &uuid::Uuid,
        conn: &mut Connection,
    ) -> Result<Vec<Self>, Error> {
        app_review_signatures::dsl::app_review_signatures
            .filter(app_review_signatures::user_id.eq(user_id.to_string()))
            .get_results(conn)
    }

    pub fn find_latest_sequence_number(
        source_id: &str,
        app_bundle_id: &str,
        conn: &mut Connection,
    ) -> Result<i32, Error> {
        app_review_signatures::dsl::app_review_signatures
            .filter(app_review_signatures::source_id.eq(source_id.to_string()))
            .filter(app_review_signatures::app_bundle_id.eq(app_bundle_id.to_string()))
            .select(diesel::dsl::max(app_review_signatures::sequence_number))
            .first::<Option<i32>>(conn)
            .map(|n| n.unwrap_or(0))
    }
}

impl AppReviewSignature {
    pub fn new(req: &AppReviewSignatureRequest, user_id: &uuid::Uuid) -> Self {
        AppReviewSignature {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            status: AppReviewStatus::Published.into(),
            sequence_number: -1,
            source_id: req.source_identifier.clone(),
            app_bundle_id: req.app_bundle_id.clone(),
            app_version: Some(req.version_number.clone()),
            review_rating: Some(req.review_rating.into()),
            signature: None,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}
