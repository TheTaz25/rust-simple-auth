use bb8::PooledConnection;
use diesel::{delete, ExpressionMethods};
use diesel::query_dsl::methods::FilterDsl;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use diesel_async::RunQueryDsl;
use diesel::result::Error::DatabaseError;
use diesel::associations::HasTable;

use crate::models::otp::{OtpInternal, OtpEnum, InsertableOtp};
use crate::{models::otp::NewOtp, utils::error::Fault};

type Conn<'a> = PooledConnection<'a, AsyncDieselConnectionManager<AsyncPgConnection>>;

pub async fn i_otp(connection: &mut Conn<'_>, new_otp: NewOtp, code_type: OtpEnum) -> Result<(), Fault> {
  use crate::schema::otp;

  let to_insert = InsertableOtp {
    code: new_otp.code,
    user: new_otp.user,
    code_type,
  };

  diesel::insert_into(otp::table)
    .values(to_insert)
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

async fn d_code(connection: &mut Conn<'_>, otp_code: &String) {
  use crate::schema::otp::dsl::*;

  let _ = delete(otp.filter(code.eq(otp_code)))
  .execute(connection)
  .await;
}

pub async fn q_check_registration_code(connection: &mut Conn<'_>, otp_code: &String) -> Result<(), Fault> {
  use crate::schema::otp::dsl::*;

  let found_code = otp
    .filter(code.eq(otp_code))
    .first::<OtpInternal>(connection)
    .await;

  match found_code {
    Ok(c) => {
      if c.code_type != OtpEnum::REGISTER {
        return Err(Fault::RegistrationCodeInvalid);
      }

      d_code(connection, otp_code).await;

      return Ok(());
    }
    _ => return Err(Fault::RegistrationCodeInvalid)
  }
}

pub async fn q_check_password_code(connection: &mut Conn<'_>, otp_code: &String) -> Result<OtpInternal, Fault> {
  use crate::schema::otp::dsl::*;

  let found_code = otp
    .filter(code.eq(otp_code))
    .first::<OtpInternal>(connection).await;

  match found_code {
    Ok(c) => {
      if c.code_type != OtpEnum::PWRESET {
        return Err(Fault::PasswordCodeInvalid);
      }

      let _ = d_code(connection, otp_code).await;

      return Ok(c);
    }
    _ => return Err(Fault::PasswordCodeInvalid)
  }
}