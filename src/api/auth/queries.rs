use bb8::PooledConnection;
use diesel::{ExpressionMethods, SelectableHelper};
use diesel::associations::HasTable;
use diesel::dsl::count_star;
use diesel::query_dsl::methods::{FilterDsl,SelectDsl};
use diesel_async::RunQueryDsl;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use uuid::Uuid;

use crate::models::user::{User, NewUser, UserInfo};
use crate::utils::error::Fault;

type Conn<'a> = PooledConnection<'a, AsyncDieselConnectionManager<AsyncPgConnection>>;

pub async fn q_get_all_users(connection: &mut Conn<'_>) -> Result<Vec<UserInfo>, Fault> {
  use crate::schema::users::dsl::*;

  let res = users::table()
    .load::<User>( connection)
    .await
    .or_else(|_| Err(Fault::DatabaseConnection))?;

  let user_info_mapped = res.into_iter()
    .map(|u| UserInfo::from(u))
    .collect::<Vec<UserInfo>>();

  Ok(user_info_mapped)
}

pub async fn q_does_user_exist(connection: &mut Conn<'_>, _username: &String) -> Result<(), ()> {
  use crate::schema::users::dsl::*;

  let results: i64 = users
    .filter(username.eq(_username))
    .select(count_star())
    .first(connection)
    .await.ok().unwrap();

  if results != 0 {
    return Ok(());
  }
  Err(())
}

pub async fn q_insert_user(connection: &mut Conn<'_>, to_insert: NewUser<'_>) -> Result<(), Fault> {
  use crate::schema::users;

  diesel::insert_into(users::table)
    .values(to_insert)
    .execute(connection)
    .await
    .or_else(|_| Err(Fault::Diesel))
    .and_then(|_| Ok(()))
}

pub async fn q_get_user_by_name(connection: &mut Conn<'_>, _username: &String) -> Result<User, Fault> {
  use crate::schema::users::dsl::*;

  users
    .filter(username.eq(_username))
    .select(User::as_select())
    .first::<User>(connection)
    .await
    .or_else(|_| Err(Fault::NotFound(String::from("User"))))
}

pub async fn q_get_user_by_id(connection: &mut Conn<'_>, _user_id: Uuid) -> Result<User, Fault> {
  use crate::schema::users::dsl::*;

  users
    .filter(user_id.eq(_user_id))
    .select(User::as_select())
    .first::<User>(connection)
    .await
    .or_else(|_| Err(Fault::NotFound(String::from("User"))))
}
