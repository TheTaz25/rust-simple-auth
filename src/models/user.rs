use axum::http::StatusCode;
use diesel::prelude::*;
use uuid::Uuid;
use crate::schema::users;

#[derive(serde::Serialize, serde::Deserialize, Selectable, Queryable, Clone)]
pub struct User {
  pub user_id: Uuid,
  pub username: String,
  pub password: String,
  pub admin: Option<bool>,
}

impl User {
  pub fn verify_password(&self, password: String) -> Result<(), StatusCode> {
    let success = bcrypt::verify(password, &self.password);
    match success {
      Ok(_) => Ok(()),
      _ => Err(StatusCode::FORBIDDEN)
    }
  }
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
  pub user_id: &'a Uuid,
  pub username: &'a str,
  pub password: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewAdmUser {
  pub user_id: Uuid,
  pub username: String,
  pub password: String,
  pub admin: Option<bool>,
}