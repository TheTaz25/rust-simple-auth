use axum::{Router, routing::{post, delete}, middleware, http::StatusCode, Json, extract::{State, Path}};

use crate::{state::AppState, middleware::authorized::admin_guard, utils::error::Fault, models::otp::{NewOtp, OtpExternal}};

use super::queries::{i_otp, q_otp_list, d_otp};

async fn create_otp(
  State(state): State<AppState>,
  Json(new_otp): Json<NewOtp>,
) -> Result<StatusCode, Fault> {
  let mut connection = state.pool.get_connection().await?.connection;

  let _ = i_otp(&mut connection, new_otp).await?;

  Ok(StatusCode::OK)
}

async fn list_otp(
  State(state): State<AppState>,
) -> Result<(StatusCode, Json<Vec<OtpExternal>>), Fault> {
  let mut connection = state.pool.get_connection().await?.connection;

  let otp_list = q_otp_list(&mut connection).await?;

  let mapped: Vec<OtpExternal> = otp_list.into_iter().map(|otp| otp.into()).collect();

  Ok((StatusCode::OK, Json(mapped)))
}

async fn delete_otp (
  State(state): State<AppState>,
  Path(id): Path<i32>
) -> Result<StatusCode, Fault> {
  let mut connection = state.pool.get_connection().await?.connection;

  d_otp(&mut connection, id).await?;

  Ok(StatusCode::OK)
}

pub fn router(state: AppState) -> Router<AppState> {
  Router::new()
    .route("/otp", 
      post(create_otp)
      .get(list_otp)
      .layer(middleware::from_fn_with_state(state.clone(), admin_guard))
    )
    .route("/otp/:id",
      delete(delete_otp)
      .layer(middleware::from_fn_with_state(state.clone(), admin_guard))
    )
}