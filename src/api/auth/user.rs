use axum::{
  Router,
  extract::{State,Json,Path,Extension},
  routing::{get,post},
  http::StatusCode,
  middleware,
  // debug_handler,
};
use diesel::{ExpressionMethods, dsl::count_star, SelectableHelper, associations::HasTable};
use serde::{Serialize,Deserialize};
use uuid::Uuid;
use diesel::query_dsl::methods::{FilterDsl,SelectDsl};
use diesel_async::RunQueryDsl;

use crate::{state::AppState, middleware::authorized::logged_in_guard, models::user::NewUser};
use crate::api::auth::session::TokenPair;
use crate::api::auth::password::hash_password;
use crate::models::user::User;

#[derive(Clone)]
pub struct UserList {
  list: Vec<User>
}

impl UserList {
  pub fn new() -> Self {
    UserList { list: vec![] }
  }

  pub fn get_all(&self) -> Vec<User> {
    self.list.to_vec()
  }

  pub fn add(&mut self, user_to_add: User) {
    self.list.push(user_to_add)
  }

  pub fn find(&self, name: &str) -> Result<&User, (StatusCode, String)> {
    self.list.iter().find(|user| user.username == name).ok_or((StatusCode::NOT_FOUND, String::from("User unknown")))
  }

  pub fn exists(&self, name: &str) -> bool {
    self.list.iter().any(|u| u.username == name)
  }
}

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
  use crate::schema::users::dsl::*;

  let mut connection = state
    .pool
    .get()
    .await
    .or_else(|_| Err(StatusCode::INTERNAL_SERVER_ERROR))?;

  let query_result = users::table()
    .load(&mut connection)
    .await
    .or_else(|_| Err(StatusCode::INTERNAL_SERVER_ERROR))?;

  let response = UserListResponse {
    users: query_result
  };

  Ok((StatusCode::OK, Json(response)))
}

async fn add_user(
  State(state): State<AppState>,
  Json(new_user): Json<NewUserBody>,
) -> Result<StatusCode, StatusCode> {
  use crate::schema::users::dsl::*;
  use crate::schema::users;

  let mut connection = state
    .pool
    .get()
    .await
    .or_else(|_| Err(StatusCode::INTERNAL_SERVER_ERROR))?;

  let results: i64 = users
    .filter(username.eq(&new_user.username))
    .select(count_star())
    .first(&mut connection)
    .await.ok().unwrap();

  if results != 0 {
    return Err(StatusCode::CONFLICT);
  }

  let hashed = hash_password(new_user.password).or_else(|_| Err(StatusCode::INTERNAL_SERVER_ERROR))?;
  let new_user_ = NewUser {
    username: &new_user.username,
    user_id: &Uuid::new_v4(),
    password: &hashed
  };

  diesel::insert_into(users::table)
    .values(new_user_)
    .execute(&mut connection)
    .await
    .or_else(|_| Err(StatusCode::INTERNAL_SERVER_ERROR))?;

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
  use crate::schema::users::dsl::*;

  let mut connection = state
    .pool
    .get()
    .await
    .or_else(|_| Err(StatusCode::FORBIDDEN))?;

  // find user by username
  let result: User = users
    .filter(username.eq(&user_data.username))
    .select(User::as_select())
    .first::<User>(&mut connection)
    .await
    .or_else(|_| Err(StatusCode::NOT_FOUND))?;
  
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

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/users", get(get_all_users))
    .route("/auth/register", post(add_user))
    .route("/auth/login", post(login_user))
    .route("/auth/test", get(test_user_authorized).layer(middleware::from_fn(logged_in_guard)))
    .route("/auth/refresh/:refresh_token", get(refresh_user_token))
}