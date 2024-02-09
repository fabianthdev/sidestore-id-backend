pub mod app_review;
pub mod oauth_authorization;
pub mod user;

use diesel::result::Error;

use crate::db::Connection;

pub trait DbModel {
    fn insert(&mut self, conn: &mut Connection) -> Result<Self, Error>
    where
        Self: std::marker::Sized;

    fn update(&mut self, conn: &mut Connection) -> Result<Self, Error>
    where
        Self: std::marker::Sized;

    fn delete(&mut self, conn: &mut Connection) -> Result<(), Error>;
}

macro_rules! db_model {
    ($dsl:expr, $id:expr, $($t:ty), + $(,)?) => ($(
        impl DbModel for $t {
            fn insert(&mut self, conn: &mut Connection) -> Result<Self, Error> {
                match diesel::insert_into($dsl).values(self.clone()).execute(conn) {
                    Ok(_) => Ok(self.clone()),
                    Err(e) => {
                        error!("Error inserting db model: {:?}", e);
                        Err(e)
                    }
                }
            }

            fn update(&mut self, conn: &mut Connection) -> Result<Self, Error> {
                self.updated_at = Utc::now().naive_utc();

                match diesel::update($dsl.filter($id.eq(self.id.to_string()))).set(self.clone()).execute(conn) {
                    Ok(_) => Ok(self.clone()),
                    Err(e) => {
                        error!("Error updating db model: {:?}", e);
                        Err(e)
                    }
                }
            }

            fn delete(&mut self, conn: &mut Connection) -> Result<(), Error> {
                match diesel::delete($dsl.filter($id.eq(self.id.to_string()))).execute(conn) {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        error!("Error deleting db model: {:?}", e);
                        Err(e)
                    }
                }
            }
        }
    )+)
}

pub(crate) use db_model;
