use std::{io::Write, str::FromStr};

use diesel::{backend::Backend, deserialize::FromSql, pg::Pg, prelude::*, serialize::{IsNull, ToSql}, AsExpression, FromSqlRow};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::schema::{otp, sql_types::OtpType};

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Eq, Clone, Serialize, Deserialize)]
#[diesel(sql_type = OtpType)]
pub enum OtpEnum {
  REGISTER,
  PWRESET,
}

impl ToSql<OtpType, Pg> for OtpEnum {
  fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, Pg>) -> diesel::serialize::Result {
    match *self {
      OtpEnum::PWRESET => out.write_all(b"PW_RESET")?,
      OtpEnum::REGISTER => out.write_all(b"REGISTER")?,
    }
    Ok(IsNull::No)
  }
}

impl FromSql<OtpType, Pg> for OtpEnum {
  fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
    match bytes.as_bytes() {
      b"PW_RESET" => Ok(OtpEnum::PWRESET),
      b"REGISTER" => Ok(OtpEnum::REGISTER),
      _ => Err("Unrecognized enum variant".into()),
    }
  }
}

impl FromStr for OtpEnum {
  type Err = ();
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "REGISTER" => Ok(OtpEnum::REGISTER),
      "PW_RESET" => Ok(OtpEnum::PWRESET),
      _ => Err(()),
    }
  }
}

impl ToString for OtpEnum {
  fn to_string(&self) -> String {
      match self {
        OtpEnum::PWRESET => "PW_RESET".into(),
        OtpEnum::REGISTER => "REGISTER".into(),
      }
  }
}

// impl Serialize for OtpEnum {
//   fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//       where
//           S: serde::Serializer {
//             match *self {
//               _ => serializer.serialize_str(self.to_string().as_str())
//             }
//   }
// }

#[derive(Queryable, Selectable, Clone)]
#[diesel(table_name = otp)]
pub struct OtpInternal {
  pub id: i32,
  pub code: String,
  pub code_type: OtpEnum,
  pub user: Option<Uuid>,
}

impl From<OtpExternal> for OtpInternal {
  fn from(value: OtpExternal) -> Self {
      Self {
        id: value.id,
        code: value.code,
        code_type: OtpEnum::from_str(value.code_type.as_str()).ok().unwrap(),
        user: value.user,
      }
  }
}

#[derive(Serialize, Deserialize, Queryable)]
#[serde(rename_all(serialize="camelCase"))]
pub struct OtpExternal {
  pub id: i32,
  pub code: String,
  pub code_type: String,
  pub user: Option<Uuid>,
}

impl From<OtpInternal> for OtpExternal {
  fn from(value: OtpInternal) -> Self {
      Self {
        code: value.code,
        code_type: value.code_type.to_string(),
        id: value.id,
        user: value.user,
      }
  }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewOtp {
  pub code: String,
  pub user: Option<Uuid>,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = otp)]
#[serde(rename_all = "camelCase")]
pub struct InsertableOtp {
  pub code: String,
  pub user: Option<Uuid>,
  pub code_type: OtpEnum
}
