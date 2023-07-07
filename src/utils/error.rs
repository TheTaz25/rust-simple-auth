use axum::{response::IntoResponse, http::StatusCode, Json};
use serde_json::json;

#[derive(Clone)]
pub enum Fault {
  Diesel,
  DatabaseConnection,
  NotLoggedIn,
  UuidConversion,
  Unexpected,
  NotFound(String),
  AlreadyExists(String),
  Unallowed,
}

impl IntoResponse for Fault {
  fn into_response(self) -> axum::response::Response {
      let x = self.clone();
      let (status, error_message) = match x {
        Fault::DatabaseConnection => (StatusCode::INTERNAL_SERVER_ERROR, "Unknown error".to_string()),
        Fault::NotLoggedIn => (StatusCode::UNAUTHORIZED, "Please log into the application in order to execute this function".to_string()),
        Fault::UuidConversion => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse data due to unexpected format".to_string()),
        Fault::Unexpected => (StatusCode::INTERNAL_SERVER_ERROR, "Unknown error".to_string()),
        Fault::Diesel => (StatusCode::INTERNAL_SERVER_ERROR, "Unknown Error".to_string()),
        Fault::NotFound(thing) => (StatusCode::NOT_FOUND, format!("{thing} not found").to_string()),
        Fault::AlreadyExists(thing) => (StatusCode::CONFLICT, format!("{thing} does already exist").to_string()),
        Fault::Unallowed => (StatusCode::FORBIDDEN, "Insufficient permissions".to_string())
      };

      let body = Json(json!({
        "error": error_message
      }));

      (status, body).into_response()
  }
}