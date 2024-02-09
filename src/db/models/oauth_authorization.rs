use std::ops::Deref;

use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::result::Error;
use serde::{Deserialize, Serialize};

use crate::db::models::user::User;
use crate::db::schema::oauth_authorizations;
use crate::db::Connection;

#[derive(
    Identifiable,
    Insertable,
    Associations,
    Queryable,
    Selectable,
    PartialEq,
    Debug,
    Clone,
    Serialize,
    Deserialize,
)]
#[diesel(belongs_to(User))]
#[diesel(primary_key(user_id, client_id))]
#[diesel(table_name = oauth_authorizations)]
pub struct OAuthAuthorization {
    pub user_id: String,
    pub client_id: String,
    #[serde(skip_serializing)]
    pub created_at: NaiveDateTime,
    #[serde(skip_serializing)]
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserDTO {
    pub user_id: String,
    pub client_id: String,
}

impl OAuthAuthorization {
    pub fn new(user: &User, client_id: &str) -> Self {
        let now = Utc::now().naive_utc();

        Self {
            user_id: user.id.to_string(),
            client_id: client_id.to_string(),

            created_at: now,
            updated_at: now,
        }
    }
}

impl OAuthAuthorization {
    pub fn insert(&mut self, conn: &mut Connection) -> Result<(), Error> {
        diesel::insert_into(oauth_authorizations::dsl::oauth_authorizations)
            .values(self.deref())
            .execute(conn)
            .map(|_| ())
    }

    pub fn delete(&mut self, conn: &mut Connection) -> Result<(), Error> {
        diesel::delete(self.deref()).execute(conn).map(|_| ())
    }
}

impl User {
    pub fn authorization_for_oauth_client(
        &self,
        client_id: &str,
        conn: &mut Connection,
    ) -> Option<OAuthAuthorization> {
        OAuthAuthorization::belonging_to(self)
            .select(OAuthAuthorization::as_select())
            .filter(oauth_authorizations::client_id.eq(client_id))
            .first(conn)
            .ok()
    }

    pub fn has_authorized_oauth_client(&self, client_id: &str, conn: &mut Connection) -> bool {
        self.authorization_for_oauth_client(client_id, conn)
            .is_some()
    }

    pub fn save_oauth_client_authorization(
        &mut self,
        client_id: &str,
        conn: &mut Connection,
    ) -> Result<(), Error> {
        if self.has_authorized_oauth_client(client_id, conn) {
            return Ok(());
        }

        OAuthAuthorization::new(&self, client_id).insert(conn)
    }

    pub fn remove_oauth_client_authorization(
        &mut self,
        client_id: &str,
        conn: &mut Connection,
    ) -> Result<(), Error> {
        if let Some(mut authorization) = self.authorization_for_oauth_client(client_id, conn) {
            authorization.delete(conn)?;
        }

        Ok(())
    }
}
