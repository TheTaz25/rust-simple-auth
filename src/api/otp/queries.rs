use bb8::PooledConnection;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use diesel_async::RunQueryDsl;
use diesel::result::Error::DatabaseError;
use diesel::associations::HasTable;

use crate::models::otp::OtpInternal;
use crate::{models::otp::NewOtp, utils::error::Fault};

type Conn<'a> = PooledConnection<'a, AsyncDieselConnectionManager<AsyncPgConnection>>;

pub async fn i_otp(connection: &mut Conn<'_>, new_otp: NewOtp) -> Result<(), Fault> {
  use crate::schema::otp;

  diesel::insert_into(otp::table)
    .values(new_otp)
    .execute(connection)
    .await
    .or_else(|diesel_error| {
      match diesel_error {
        DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _) => Err(Fault::AlreadyExists("code".to_owned())),
        _ => Err(Fault::Diesel)
      }
    })
    .and_then(|_| Ok(()))
}

pub async fn q_otp_list(connection: &mut Conn<'_>) -> Result<Vec<OtpInternal>, Fault> {
  use crate::schema::otp::dsl::*;

  let res = otp::table()
    .load::<OtpInternal>(connection)
    .await
    .or_else(|_| Err(Fault::Diesel))?;

  Ok(res)
}