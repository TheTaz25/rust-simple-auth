use axum::http::StatusCode;
use bb8::PooledConnection;
use diesel::{ExpressionMethods, SelectableHelper};
use diesel::associations::HasTable;
use diesel::dsl::count_star;
use diesel::query_dsl::methods::{FilterDsl,SelectDsl};
use diesel_async::RunQueryDsl;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};

use crate::models::user::{User, NewUser};

type Conn<'a> = PooledConnection<'a, AsyncDieselConnectionManager<AsyncPgConnection>>;

pub async fn q_get_all_users(connection: &mut Conn<'_>) -> Result<Vec<User>, StatusCode> {
  use crate::schema::users::dsl::*;

  let res = users::table()
    .load::<User>( connection)
    .await
    .or_else(|_| Err(StatusCode::INTERNAL_SERVER_ERROR))?;

  Ok(res)
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

pub async fn q_insert_user(connection: &mut Conn<'_>, to_insert: NewUser<'_>) -> Result<(), StatusCode> {
  use crate::schema::users;

  diesel::insert_into(users::table)
    .values(to_insert)
    .execute(connection)
    .await
    .or_else(|_| Err(StatusCode::INTERNAL_SERVER_ERROR))
    .and_then(|_| Ok(()))
}

pub async fn q_get_user_by_name(connection: &mut Conn<'_>, _username: &String) -> Result<User, StatusCode> {
  use crate::schema::users::dsl::*;

  users
    .filter(username.eq(_username))
    .select(User::as_select())
    .first::<User>(connection)
    .await
    .or_else(|_| Err(StatusCode::NOT_FOUND))
}
