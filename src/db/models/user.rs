use log::error;
use chrono::{Utc, NaiveDateTime};
use serde::{Deserialize, Serialize};
use diesel::{Queryable, Insertable, AsChangeset, RunQueryDsl, QueryDsl, ExpressionMethods};
use diesel::result::Error;

use crate::db::Connection;
use crate::db::schema::users;


#[derive(Serialize, Deserialize, Debug, Clone, Queryable, Insertable, AsChangeset)]
#[diesel(table_name = users)]
pub struct User {
    #[serde(default, skip_serializing)]
    pub id: String,
    pub email: String,
    pub username: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash: String,
    #[serde(skip_serializing)]
    pub created_at: NaiveDateTime,
    #[serde(skip_serializing)]
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserDTO {
    pub email: String,
    pub password: String,
    pub username: Option<String>,
}

impl User {
    pub fn new(email: &str, password_hash: &str) -> Self {
        let now = Utc::now().naive_utc();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: now,
            updated_at: now,

            email: email.to_string(),
            password_hash: password_hash.to_string(),

            username: None,
        }
    }
}

impl User {
    pub fn insert(&mut self, conn: &mut Connection) -> Result<Self, Error> {
        match diesel::insert_into(users::dsl::users).values(self.clone()).execute(conn) {
            Ok(_) => Ok(self.clone()),
            Err(e) => {
                error!("Error inserting user: {:?}", e);
                Err(e)
            }
        }
    }

    pub fn find_by_id(id: &uuid::Uuid, conn: &mut Connection) -> Result<Self, Error> {
        users::dsl::users
            .find(id.to_string())
            .get_result::<User>(conn)
    }

    pub fn find_by_email(email: &str, conn: &mut Connection) -> Result<Self, Error> {
        users::dsl::users
            .filter(users::email.eq(email))
            .get_result::<User>(conn)
    }

    pub async fn update(&mut self, conn: &mut Connection) -> Result<Self, Error> {
        self.updated_at = Utc::now().naive_utc();

        match diesel::update(users::dsl::users).set(self.clone()).get_result::<User>(conn) {
            Ok(u) => Ok(u),
            Err(e) => {
                error!("Error updating user: {:?}", e);
                Err(e)
            }
        }
    }

    pub async fn delete(&mut self, conn: &mut Connection) -> Result<(), Error> {
        match diesel::delete(users::dsl::users.find(self.id.clone())).execute(conn) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Error deleting user: {:?}", e);
                Err(e)
            }
        }
    }
}