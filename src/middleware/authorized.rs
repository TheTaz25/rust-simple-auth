use axum::{http::{Request, StatusCode}, response::Response, middleware::Next};

use crate::utils::parser::get_authorization_as_uuid;

pub async fn logged_in_guard<B>(
  mut req: Request<B>,
  next: Next<B>,
) -> Result<Response, StatusCode> {
  let auth_token = get_authorization_as_uuid(&req.headers());

  if let Some(auth_token) = auth_token {
    req.extensions_mut().insert(auth_token);
    Ok(next.run(req).await)
  } else {
    Err(StatusCode::UNAUTHORIZED)
  }
}