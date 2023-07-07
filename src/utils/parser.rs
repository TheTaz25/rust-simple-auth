use axum::http::{HeaderMap, header};

use super::error::Fault;

pub fn get_authorization_as_uuid(headers: &HeaderMap) -> Result<String, Fault> {
  let auth_string = headers
    .get(header::AUTHORIZATION);

  if let Some(st) = auth_string {
    return Ok(st.to_str().unwrap().to_string());
  }
  Err(Fault::NotLoggedIn)
}

