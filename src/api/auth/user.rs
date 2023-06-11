use axum::{
  Router,
  extract::{State,Json,Path,Extension},
  routing::{get,post},
  http::StatusCode,
  middleware,
  // debug_handler,
};
use serde::{Serialize,Deserialize};
use uuid::Uuid;

use crate::{state::AppState, middleware::authorized::logged_in_guard, models::user::NewUser, api::auth::queries::{q_get_all_users, q_does_user_exist, q_get_user_by_name}};
use crate::api::auth::session::TokenPair;
use crate::api::auth::password::hash_password;
use crate::models::user::User;

use super::queries::q_insert_user;

#[derive(Serialize)]
struct UserListResponse {
  users: Vec<User>
}

#[derive(Serialize)]
struct UserResponse {
  user: User
}

#[derive(Deserialize)]
struct NewUserBody {
  username: String,
  password: String,
}

// TODO: Only callable by admins
async fn get_all_users(
  State(state): State<AppState>
) -> Result<(StatusCode, Json<UserListResponse>), StatusCode> {
  let mut connection = state
    .pool.get_connection().await?.connection;

  let query_result = q_get_all_users(&mut connection).await?;

  let response = UserListResponse {
    users: query_result
  };

  Ok((StatusCode::OK, Json(response)))
}

// TODO: Extend for registration-code
async fn add_user(
  State(state): State<AppState>,
  Json(new_user): Json<NewUserBody>,
) -> Result<StatusCode, StatusCode> {
  let mut connection = state.pool.get_connection().await?.connection;

  let does_exist = q_does_user_exist(&mut connection, &new_user.username).await;

  if does_exist.is_ok() {
    return Err(StatusCode::CONFLICT);
  }

  let hashed = hash_password(new_user.password)
    .or_else(|_| Err(StatusCode::INTERNAL_SERVER_ERROR))?;

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
) -> Result<(StatusCode, Json<LoginResponse>), StatusCode> {
  let mut connection = state
    .pool.get_connection().await?.connection;

  // find user by username
  let result: User = q_get_user_by_name(&mut connection, &user_data.username).await?;
  
  // let y = state.pool.with_connection(|connection| async move {
  //   let o = q_get_all_users(&mut connection.as_mut().connection).await?;
  //   Ok(o)
  // }).await?;
  
  // verify user password
  result.verify_password(user_data.password)?;
  
  // generate token pair, save it
  let token_pair = TokenPair::new(&result.user_id);
  state.redis.save_token_pair_for_user(&token_pair).await?;


  Ok((StatusCode::OK, Json(LoginResponse { tokens: token_pair })))
}

#[derive(Serialize)]
struct AllTokensResponse {
  tokens: Vec<TokenPair>
}

async fn test_user_authorized(
  Extension(user): Extension<User>,
) -> Result<StatusCode, StatusCode> {
  println!("{}", user.user_id.to_string());
  Ok(StatusCode::OK)
}

async fn refresh_user_token(
  State(state): State<AppState>,
  Path(refresh_token): Path<Uuid>,
) -> Result<(StatusCode, Json<LoginResponse>), StatusCode> {
  let (user_id, access_token) = state.redis.invalidate_refresh_token_and_get_result(refresh_token).await?;
  state.redis.clear_token(&access_token).await?;

  // generate new pair of tokens, save it
  let user_uuid = Uuid::parse_str(&user_id).or_else(|_| Err(StatusCode::INTERNAL_SERVER_ERROR))?;
  let token_pair = TokenPair::new(&user_uuid);

  state.redis.save_token_pair_for_user(&token_pair).await?;

  Ok((StatusCode::OK, Json(LoginResponse { tokens: token_pair })))
}

// pub fn router(appState: AppState) -> Router<AppState> {
pub fn router(state: AppState) -> Router<AppState> {
  Router::new()
    .route("/users", get(get_all_users))
    .route("/auth/register", post(add_user))
    .route("/auth/login", post(login_user))
    .route("/auth/test", get(test_user_authorized).layer(middleware::from_fn_with_state(state.clone(), logged_in_guard)))
    .route("/auth/refresh/:refresh_token", get(refresh_user_token))
}