use std::str::FromStr;

use axum::{
  Router,
  extract::{State,Json,Path,Extension},
  routing::{get,post},
  http::{StatusCode,HeaderMap},
  middleware,
  // debug_handler,
};
use serde::{Serialize,Deserialize};
use uuid::Uuid;

use crate::{state::AppState, middleware::authorized::logged_in_guard, models::user::{NewUser, UserInfo}, api::{auth::queries::{q_does_user_exist, q_get_user_by_name}, otp::queries::{q_check_registration_code, q_check_password_code}}, utils::{error::Fault, parser::get_authorization_as_uuid}};
use crate::api::auth::session::TokenPair;
use crate::api::auth::password::hash_password;
use crate::models::user::User;

use super::queries::{q_insert_user, u_set_user_password, q_get_user_by_id};

#[derive(Serialize)]
struct UserResponse {
  user: UserInfo
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
struct NewUserBody {
  username: String,
  password: String,
  registration_code: String,
}

async fn add_user(
  State(state): State<AppState>,
  Json(new_user): Json<NewUserBody>,
) -> Result<StatusCode, Fault> {
  let mut connection = state.pool.get_connection().await?.connection;

  let does_exist = q_does_user_exist(&mut connection, &new_user.username).await;

  if does_exist.is_ok() {
    return Err(Fault::AlreadyExists(String::from("User")));
  }

  q_check_registration_code(&mut connection, &new_user.registration_code).await?;

  let hashed = hash_password(new_user.password)
    .or_else(|_| Err(Fault::Unexpected))?;

  let new_user_ = NewUser {
    username: &new_user.username,
    user_id: &Uuid::new_v4(),
    password: &hashed
  };

  q_insert_user(&mut connection, new_user_).await?;

  Ok(StatusCode::CREATED)
}

#[derive(Deserialize)]
struct LoginBody {
  username: String,
  password: String,
}

#[derive(Serialize)]
struct LoginResponse {
  tokens: TokenPair
}
async fn login_user(
  State(state): State<AppState>,
  Json(user_data): Json<LoginBody>
) -> Result<(StatusCode, Json<LoginResponse>), Fault> {
  let mut connection = state
    .pool.get_connection().await?.connection;

  // find user by username
  let result: User = q_get_user_by_name(&mut connection, &user_data.username).await?;
  
  // let y = state.pool.with_connection(|connection| async move {
  //   let o = q_get_all_users(&mut connection.as_mut().connection).await?;
  //   Ok(o)
  // }).await?;
  if result.blocked.is_some_and(|b| b == true) {
    return Err(Fault::UserBlocked);
  }
  
  // verify user password
  result.verify_password(user_data.password)?;
  
  // generate token pair, save it
  let token_pair = TokenPair::new(&result.user_id);
  state.redis.save_token_pair_for_user(&token_pair).await?;


  Ok((StatusCode::OK, Json(LoginResponse { tokens: token_pair })))
}

async fn refresh_user_token(
  State(state): State<AppState>,
  Path(refresh_token): Path<Uuid>,
) -> Result<(StatusCode, Json<LoginResponse>), Fault> {
  let (user_id, access_token) = state.redis.invalidate_refresh_token_and_get_result(refresh_token).await?;
  state.redis.clear_token(&access_token).await?;

  // generate new pair of tokens, save it
  let user_uuid = Uuid::parse_str(&user_id).or_else(|_| Err(Fault::UuidConversion))?;
  let token_pair = TokenPair::new(&user_uuid);

  state.redis.save_token_pair_for_user(&token_pair).await?;

  Ok((StatusCode::OK, Json(LoginResponse { tokens: token_pair })))
}

async fn get_user_info (
  Extension(user): Extension<User>,
) -> Result<(StatusCode, Json<UserResponse>), Fault> {
  Ok((StatusCode::OK, Json(UserResponse { user: UserInfo::from(user) })))
}

async fn logout_user (
  State(state): State<AppState>,
  headers: HeaderMap,
) -> Result<StatusCode, Fault> {
  let auth_token = get_authorization_as_uuid(&headers)?;
  state.redis.invalidate_session_by_access_token(Uuid::from_str(&auth_token).ok().unwrap()).await?;
  Ok(StatusCode::OK)
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
struct UpdatePasswordByPasswordBody {
  old_password: String,
  new_password: String,
}

async fn reset_password_by_password (
  State(state): State<AppState>,
  Extension(mut user): Extension<User>,
  Json(body): Json<UpdatePasswordByPasswordBody>
) -> Result<StatusCode, Fault> {
  let mut connection = state.pool.get_connection().await?.connection;

  user.verify_password(body.old_password)?;
  user.set_password(body.new_password)?;

  u_set_user_password(&mut connection, &user).await?;

  Ok(StatusCode::OK)
}

#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
struct UpdatePasswordByOtp {
  otp_code: String,
  new_password: String,
}

async fn reset_password_by_otp (
  State(state): State<AppState>,
  Json(body): Json<UpdatePasswordByOtp>
) -> Result<StatusCode, Fault> {
  let mut connection = state.pool.get_connection().await?.connection;

  // get InternalOtp by otp_code
  let otp = q_check_password_code(&mut connection, &body.otp_code).await?;
  // get the associated user by this otp
  let mut user = q_get_user_by_id(&mut connection, otp.user.unwrap()).await?;
  // set new password on user struct
  let _ = user.set_password(body.new_password)?;
  // update password in database
  u_set_user_password(&mut connection, &user).await?;

  Ok(StatusCode::OK)
}

pub fn router(state: AppState) -> Router<AppState> {
  Router::new()
    .route("/auth/self", get(get_user_info).layer(middleware::from_fn_with_state(state.clone(), logged_in_guard)))
    .route("/auth/register", post(add_user))
    .route("/auth/login", post(login_user))
    .route("/auth/refresh/{refresh_token}", get(refresh_user_token))
    .route("/auth/logout", get(logout_user).layer(middleware::from_fn_with_state(state.clone(), logged_in_guard)))
    .route("/auth/update-password-by-password", post(reset_password_by_password).layer(middleware::from_fn_with_state(state.clone(), logged_in_guard)))
    .route("/auth/update-password-by-otp", post(reset_password_by_otp))
}