use diesel::prelude::*;
use uuid::Uuid;
use crate::{schema::users, utils::error::Fault};

#[derive(serde::Serialize, serde::Deserialize, Selectable, Queryable, Clone)]
pub struct User {
  pub user_id: Uuid,
  pub username: String,
  pub password: String,
  pub admin: Option<bool>,
  pub blocked: Option<bool>
}

#[derive(serde::Serialize)]
#[serde(rename_all(serialize="camelCase"))]
pub struct UserInfo {
  pub user_id: Uuid,
  pub username: String,
  pub admin: Option<bool>,
  pub blocked: Option<bool>
}

impl From<User> for UserInfo {
  fn from(user: User) -> Self {
    UserInfo { user_id: user.user_id, username: user.username, admin: user.admin, blocked: user.blocked }
  }
}

impl User {
  pub fn verify_password(&self, password: String) -> Result<(), Fault> {
    let success = bcrypt::verify(password, &self.password);
    match success {
      Ok(_) => Ok(()),
      _ => Err(Fault::Unallowed)
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