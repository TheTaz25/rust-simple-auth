use axum::http::{HeaderMap, header};

use super::error::Fault;

pub fn get_authorization_as_uuid(headers: &HeaderMap) -> Result<String, Fault> {
  let auth_string = headers
    .get(header::AUTHORIZATION);

  if let Some(st) = auth_string {
    let x = st.to_str().or_else(|_| Err(Fault::UuidConversion))?;
    println!("{x}");
    let y = x.split(' ').nth(1);
    if let Some(token) = y {
      return Ok(token.to_string());
    }
    return Err(Fault::MalformedAuthorization);
  }
  Err(Fault::NotLoggedIn)
}

