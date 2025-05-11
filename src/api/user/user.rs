use axum::{
  middleware,
  Router,
  routing::{get,delete},
  extract::{State, Path},
  http::StatusCode,
  Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::{state::AppState, middleware::authorized::admin_guard, utils::error::Fault, api::auth::queries::q_get_all_users, models::user::UserInfo};

use super::queries::{u_set_admin_on_user, u_block_user, u_unblock_user, d_user};

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

async fn update_admin(
  State(state): State<AppState>,
  Path((user_id, is_admin)): Path<(Uuid, bool)>
) -> Result<StatusCode, Fault> {
  let mut connection = state.pool.get_connection().await?.connection;

  u_set_admin_on_user(&mut connection, user_id, is_admin).await?;

  Ok(StatusCode::OK)
}
async fn lock_user(
  State(state): State<AppState>,
  Path(user_id): Path<Uuid>,
) -> Result<StatusCode, Fault> {
  let mut connection = state.pool.get_connection().await?.connection;

  u_block_user(&mut connection, user_id).await?;

  Ok(StatusCode::OK)
}
async fn unlock_user(
  State(state): State<AppState>,
  Path(user_id): Path<Uuid>,
) -> Result<StatusCode, Fault> {
  let mut connection = state.pool.get_connection().await?.connection;

  u_unblock_user(&mut connection, user_id).await?;

  Ok(StatusCode::OK)
}
async fn delete_user(
  State(state): State<AppState>,
  Path(user_id): Path<Uuid>,
) -> Result<StatusCode, Fault> {
  let mut connection = state.pool.get_connection().await?.connection;

  d_user(&mut connection, user_id).await?;

  Ok(StatusCode::OK)
}

pub fn router(state: AppState) -> Router<AppState> {
  Router::new()
    .route("/users", 
      get(get_all_users)
      .layer(middleware::from_fn_with_state(state.clone(), admin_guard))
    )
    .route("/users/{user_id}/admin/{is_admin}",
      get(update_admin)
      .layer(middleware::from_fn_with_state(state.clone(), admin_guard))
    )
    .route("/users/{user_id}/lock",
      get(lock_user)
      .layer(middleware::from_fn_with_state(state.clone(), admin_guard))
    )
    .route("/users/{user_id}/unlock",
      get(unlock_user)
      .layer(middleware::from_fn_with_state(state.clone(), admin_guard))
    )
    .route("/users/{user_id}",
      delete(delete_user)
        .layer(middleware::from_fn_with_state(state.clone(), admin_guard))
    )
}