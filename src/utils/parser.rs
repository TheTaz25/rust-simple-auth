use axum::http::HeaderMap;
use uuid::Uuid;

pub fn get_authorization_as_uuid(headers: &HeaderMap) -> Result<Uuid, String> {
  let auth_header_value = headers.get("Authorization");
  if auth_header_value.is_none() {
    return Err("Authorization-Header is not set".to_string());
  }
  let auth_string_value = auth_header_value.unwrap().to_str().ok().unwrap();
  let auth_uuid = Uuid::parse_str(auth_string_value);
  if auth_uuid.is_ok() {
    Ok(auth_uuid.ok().unwrap())
  } else {
    Err("Failed to parse Authorziation Header".to_string())
  }
}