use axum::http::{HeaderMap, header};
use uuid::Uuid;

pub fn get_authorization_as_uuid(headers: &HeaderMap) -> Option<Uuid> {
  let auth_uuid = headers
    .get(header::AUTHORIZATION)
    .and_then(|header| header.to_str().ok())
    .and_then(|auth_value| Uuid::parse_str(auth_value).ok());

  auth_uuid
}

