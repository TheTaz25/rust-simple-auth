use axum::{
  middleware,
  Router,
  routing::get,
  extract::State,
  Extension,
  http::StatusCode,
  Json,
};
use serde::Serialize;

use crate::{state::AppState, middleware::authorized::logged_in_guard, utils::error::Fault, api::auth::queries::q_get_all_users, models::user::{User, UserInfo}};

#[derive(Serialize)]
struct UserListResponse {
  users: Vec<UserInfo>
}

async fn get_all_users(
  State(state): State<AppState>,
  Extension(user): Extension<User>,
) -> Result<(StatusCode, Json<UserListResponse>), Fault> {
  if let Some(admin) = user.admin {
    if admin == false {
      return Err(Fault::Unallowed)
    }

    let mut connection = state
    .pool.get_connection().await?.connection;

  let query_result = q_get_all_users(&mut connection).await?;

  let response = UserListResponse {
    users: query_result
  };

  return Ok((StatusCode::OK, Json(response)));
  }
  Err(Fault::Unallowed)
}

pub fn router(state: AppState) -> Router<AppState> {
  Router::new()
    .route("/users", get(get_all_users).layer(middleware::from_fn_with_state(state.clone(), logged_in_guard)))
}