use bb8::PooledConnection;
use diesel::{delete, ExpressionMethods, update};
use diesel::query_dsl::methods::FilterDsl;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use diesel_async::RunQueryDsl;
use diesel::result::Error::DatabaseError;
use diesel::associations::HasTable;

use crate::models::otp::{OtpInternal, OtpEnum};
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

pub async fn d_otp(connection: &mut Conn<'_>, otp_id: i32) -> Result<(), Fault> {
  use crate::schema::otp::dsl::*;

  let result: usize = delete(otp.filter(id.eq(otp_id)))
    .execute(connection)
    .await.or_else(|_| Err(Fault::Diesel))?;

  if result == 0 {
    return Err(Fault::NotFound("code".to_owned()));
  }

  Ok(())
}

pub async fn q_check_registration_code(connection: &mut Conn<'_>, otp_code: &String) -> Result<(), Fault> {
  use crate::schema::otp::dsl::*;

  let found_code = otp
    .filter(code.eq(otp_code))
    .first::<OtpInternal>(connection)
    .await
    .or_else(|_| Err(Fault::Diesel))?;

  if found_code.code_type == OtpEnum::REGISTER && found_code.usages_left.unwrap() > 0 {
    let usages = found_code.usages_left.unwrap_or(1);
    update(otp.filter(code.eq(otp_code)))
      .set(usages_left.eq(Some(usages - 1)))
      .execute(connection)
      .await
      .or_else(|_| Err(Fault::Diesel))?;

    return Ok(());
  }

  Err(Fault::RegistrationCodeInvalid)
}