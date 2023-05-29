use axum::{http::{Request, StatusCode}, response::Response, middleware::Next, extract::State};

use crate::{utils::parser::get_authorization_as_uuid, state::AppState};

pub async fn logged_in_guard<B>(
  State(state): State<AppState>,
  mut req: Request<B>,
  next: Next<B>,
) -> Result<Response, StatusCode> {
  let auth_token = get_authorization_as_uuid(&req.headers());

  if let Ok(auth_token) = auth_token {
    // let mut connection = state.pool.get().await.or_else(|_| Err(StatusCode::INTERNAL_SERVER_ERROR))?;



    req.extensions_mut().insert(auth_token);
    Ok(next.run(req).await)
  } else {
    Err(StatusCode::UNAUTHORIZED)
  }
}