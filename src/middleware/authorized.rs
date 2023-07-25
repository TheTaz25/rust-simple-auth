use axum::{http::Request, response::Response, middleware::Next, extract::State};

use crate::{utils::{parser::get_authorization_as_uuid, error::Fault}, state::AppState, api::auth::queries::q_get_user_by_id};

pub async fn logged_in_guard<B>(
  State(state): State<AppState>,
  mut req: Request<B>,
  next: Next<B>,
) -> Result<Response, Fault> {
  let auth_token = get_authorization_as_uuid(&req.headers());
  if let Ok(auth_token) = auth_token {
    let user_uuid = state.redis.get_user_for_access_token(&auth_token).await?;

    let mut connection = state.pool.get_connection().await?;

    let user = q_get_user_by_id(&mut connection.connection, user_uuid).await?;

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
  } else {
    Err(Fault::NotLoggedIn)
  }
}

pub async fn admin_guard<B>(
  State(state): State<AppState>,
  mut req: Request<B>,
  next: Next<B>,
) -> Result<Response, Fault> {
  let auth_token = get_authorization_as_uuid(&req.headers());
  if let Ok(auth_token) = auth_token {
    let user_uuid = state.redis.get_user_for_access_token(&auth_token).await?;

    let mut connection = state.pool.get_connection().await?;

    let user = q_get_user_by_id(&mut connection.connection, user_uuid).await?;

    if let Some(admin) = user.admin {
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