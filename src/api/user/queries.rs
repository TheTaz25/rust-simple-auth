use diesel::{prelude::*, update, delete};
use diesel_async::RunQueryDsl;
use bb8::PooledConnection;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use uuid::Uuid;

use crate::utils::error::Fault;

type Conn<'a> = PooledConnection<'a, AsyncDieselConnectionManager<AsyncPgConnection>>;

pub async fn u_set_admin_on_user(connection: &mut Conn<'_>, user_uuid: Uuid, is_admin: bool) -> Result<(), Fault> {
  use crate::schema::users::dsl::*;

  let _ = update(users.filter(user_id.eq(user_uuid)))
    .set(admin.eq(is_admin))
    .execute(connection)
    .await
    .or_else(|_| Err(Fault::Diesel))?;

  Ok(())
}

pub async fn u_block_user (connection: &mut Conn<'_>, user: Uuid) -> Result<(), Fault> {
  use crate::schema::users::dsl::*;

  let result: usize = update(users.filter(user_id.eq(user)))
    .set(blocked.eq(true))
    .execute(connection)
    .await
    .or_else(|_| Err(Fault::Diesel))?;

  println!("{} updated rows", result);

  if result == 0 {
    return Err(Fault::NotFound("user".to_owned()));
  }

  Ok(())
}

pub async fn u_unblock_user (connection: &mut Conn<'_>, user: Uuid) -> Result<(), Fault> {
  use crate::schema::users::dsl::*;

  let result: usize = update(users.filter(user_id.eq(user)))
    .set(blocked.eq(false))
    .execute(connection)
    .await
    .or_else(|_| Err(Fault::Diesel))?;

  if result == 0 {
    return Err(Fault::NotFound("user".to_owned()));
  }

  Ok(())
}

pub async fn d_user (connection: &mut Conn<'_>, user: Uuid) -> Result<(), Fault> {
  use crate::schema::users::dsl::*;

  let result: usize = delete(users.filter(user_id.eq(user)))
    .execute(connection)
    .await.or_else(|_| Err(Fault::Diesel))?;

  if result == 0 {
    return Err(Fault::NotFound("user".to_owned()));
  }

  Ok(())
}
