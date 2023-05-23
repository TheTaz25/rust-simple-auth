use diesel::prelude::*;
use uuid::Uuid;
use crate::schema::users;

#[derive(serde::Serialize, serde::Deserialize, Selectable, Queryable)]
pub struct User {
  pub user_id: Uuid,
  pub username: String,
  pub password: String,
  pub admin: Option<bool>,
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