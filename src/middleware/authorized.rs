use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};

use crate::{utils::{parser::get_authorization_as_uuid, error::Fault}, state::AppState, api::auth::queries::q_get_user_by_id};

pub async fn logged_in_guard(
  State(state): State<AppState>,
  mut req: Request<Body>,
  next: Next,
) -> Result<Response, Fault> {
  let auth_token = get_authorization_as_uuid(&req.headers());
  if let Ok(auth_token) = auth_token {
    let user_uuid = state.redis.get_user_for_access_token(&auth_token).await?;

    let mut connection = state.pool.get_connection().await?;

    let user = q_get_user_by_id(&mut connection.connection, user_uuid).await?;

    if user.blocked.is_some_and(|b| b) {
      return Err(Fault::UserBlocked);
    }

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
  } else {
    Err(Fault::NotLoggedIn)
  }
}

pub async fn admin_guard(
  State(state): State<AppState>,
  mut req: Request<Body>,
  next: Next,
) -> Result<Response, Fault> {
  let auth_token = get_authorization_as_uuid(&req.headers());
  if let Ok(auth_token) = auth_token {
    let user_uuid = state.redis.get_user_for_access_token(&auth_token).await?;

    let mut connection = state.pool.get_connection().await?;

    let user = q_get_user_by_id(&mut connection.connection, user_uuid).await?;

    if let (Some(admin), Some(blocked)) = (user.admin, user.blocked) {
      if blocked {
        return Err(Fault::UserBlocked);
      }
      if admin {
        req.extensions_mut().insert(user);
        return Ok(next.run(req).await);
      }
    }
    return Err(Fault::Unallowed)
  } else {
    Err(Fault::NotLoggedIn)
  }
}