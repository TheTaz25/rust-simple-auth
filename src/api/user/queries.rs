use diesel::{prelude::*, update};
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