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
  
  // verify user password
  result.verify_password(user_data.password)?;
  
  // generate token pair, save it
  // TODO: MOVE TO REDIS
  let mut token_list = state.token_list.lock().unwrap();
  let token_pair = TokenPair::new(&result.user_id);
  token_list.add(token_pair);

  Ok((StatusCode::OK, Json(LoginResponse { tokens: token_pair })))
}

#[derive(Serialize)]
struct AllTokensResponse {
  tokens: Vec<TokenPair>
}

async fn test_user_authorized(
  State(state): State<AppState>,
  Extension(auth_uuid): Extension<Uuid>,
) -> Result<StatusCode, StatusCode> {
  let token_list = state.token_list.lock().unwrap();
  let user_id = token_list.get_user_id_from_access_token(auth_uuid)?;
  println!("{}", user_id.to_string());
  Ok(StatusCode::OK)
}

async fn refresh_user_token(
  State(state): State<AppState>,
  Path(refresh_token): Path<Uuid>,
) -> Result<(StatusCode, Json<LoginResponse>), StatusCode> {
  let mut token_list = state.token_list.lock().unwrap();

  // Check if token is valid, extract user-id, invalidate old tokens
  token_list.refresh_token_valid(refresh_token)?;
  let user_id = token_list.get_user_id_from_refresh_token(refresh_token)?;
  token_list.remove_by_refresh_token(refresh_token);

  // generate new pair of tokens, save it
  let token_pair = TokenPair::new(&user_id);
  token_list.add(token_pair);

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