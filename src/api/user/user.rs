use axum::{
  middleware,
  Router,
  routing::{get,delete},
  extract::State,
  http::StatusCode,
  Json,
};
use serde::Serialize;

use crate::{state::AppState, middleware::authorized::admin_guard, utils::error::Fault, api::auth::queries::q_get_all_users, models::user::UserInfo};

#[derive(Serialize)]
struct UserListResponse {
  users: Vec<UserInfo>
}

async fn get_all_users(
  State(state): State<AppState>,
) -> Result<(StatusCode, Json<UserListResponse>), Fault> {
  let mut connection = state
  .pool.get_connection().await?.connection;

  let query_result = q_get_all_users(&mut connection).await?;

  let response = UserListResponse {
    users: query_result
  };

  return Ok((StatusCode::OK, Json(response)));
}

async fn update_admin() -> Result<StatusCode, Fault> {
  Err(Fault::NotImplementedYet)
}
async fn lock_user() -> Result<StatusCode, Fault> {
  Err(Fault::NotImplementedYet)
}
async fn unlock_user() -> Result<StatusCode, Fault> {
  Err(Fault::NotImplementedYet)
}
async fn delete_user() -> Result<StatusCode, Fault> {
  Err(Fault::NotImplementedYet)
}

pub fn router(state: AppState) -> Router<AppState> {
  Router::new()
    .route("/users", 
      delete(delete_user)
      .get(get_all_users)
      .layer(middleware::from_fn_with_state(state.clone(), admin_guard))
    )
    .route("/users/:user_id/admin",
      get(update_admin)
      .layer(middleware::from_fn_with_state(state.clone(), admin_guard))
    )
    .route("/users/:user_id/lock",
      get(lock_user)
      .layer(middleware::from_fn_with_state(state.clone(), admin_guard))
    )
    .route("/users/:user_id/unlock",
      get(unlock_user)
      .layer(middleware::from_fn_with_state(state.clone(), admin_guard))
    )
}