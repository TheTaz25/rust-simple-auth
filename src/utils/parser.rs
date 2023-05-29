use axum::http::{HeaderMap, header, StatusCode};
use uuid::Uuid;

pub fn get_authorization_as_uuid(headers: &HeaderMap) -> Result<Uuid, StatusCode> {
  let auth_string = headers
    .get(header::AUTHORIZATION);

  if let Some(st) = auth_string {
    return Uuid::parse_str(&st.to_str().unwrap()).or_else(|_| Err(StatusCode::UNAUTHORIZED))
  }
  Err(StatusCode::UNAUTHORIZED)
}

