use axum::{response::IntoResponse, http::StatusCode, Json};
use serde_json::json;

pub enum Fault {
  DatabaseConnectionError,
}

impl IntoResponse for Fault {
  fn into_response(self) -> axum::response::Response {
      let (status, error_message) = match self {
        Fault::DatabaseConnectionError => (StatusCode::INTERNAL_SERVER_ERROR, "Unknown error"),
      };

      let body = Json(json!({
        "error": error_message
      }));

      (status, body).into_response()
  }
}