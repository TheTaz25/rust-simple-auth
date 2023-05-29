use axum::{
  Router,
  extract::{State,Json,Path,Extension},
  routing::{get,post},
  http::StatusCode,
  middleware,
  debug_handler,
};
use diesel::{ExpressionMethods, dsl::count_star};
use serde::{Serialize,Deserialize};
use uuid::Uuid;
use diesel::query_dsl::methods::{FilterDsl,SelectDsl};
use diesel_async::RunQueryDsl;

use crate::{state::AppState, middleware::authorized::logged_in_guard, models::user::NewUser};
use crate::api::auth::session::TokenPair;
use crate::api::auth::password::hash_password;
use crate::models::user::User;

// #[derive(Clone, Serialize)]
// pub struct User {
//   user_id: Uuid,
//   username: String,
//   password: String,
//   admin: bool,
// }

// impl User {
  // pub fn new(username: String, clear_text_password: String, admin: bool) -> Self {
  //   User {
  //     user_id: Uuid::new_v4(),
  //     username,
  //     password: hash_password(clear_text_password).expect("Was not able to generate a hashed password"),
  //     admin,
  //   }
  // }
//   fn verify_password(&self, password: String) -> Result<(), (StatusCode, String)> {
//     let success = bcrypt::verify(password, &self.password).is_ok();
//     if success {
//       Ok(())
//     } else {
//       Err((StatusCode::FORBIDDEN, String::from("wrong password")))
//     }
//   }
// }

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

async fn get_all_users(
  State(state): State<AppState>
) -> (StatusCode, Json<UserListResponse>) {
  let user_list = state.user_list.lock().unwrap();
  let response = UserListResponse {
    users: user_list.get_all()
  };
  (StatusCode::OK, Json(response))
}

async fn find_user(
  State(state): State<AppState>,
  Path(name): Path<String>
) -> Result<(StatusCode, Json<UserResponse>), (StatusCode, String)> {
  let user_list = state.user_list.lock().unwrap();
  let found_user = user_list.find(&name)?;
  Ok((StatusCode::OK, Json(UserResponse { user: found_user.clone() })))
}

#[debug_handler]
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

// async fn login_user(
//   State(state): State<AppState>,
//   Json(user_data): Json<LoginBody>
// ) -> Result<(StatusCode, Json<LoginResponse>), (StatusCode, String)> {
//   let user_list = state.user_list.lock().unwrap();
//   let mut token_list = state.token_list.lock().unwrap();

//   let user = user_list.find(&user_data.username)?;
//   user.verify_password(user_data.password)?;

//   let token_pair = TokenPair::new(user.user_id);

//   token_list.add(token_pair);

//   Ok((StatusCode::OK, Json(LoginResponse { tokens: token_pair })))
// }

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
  let token_pair = TokenPair::new(user_id);
  token_list.add(token_pair);

  Ok((StatusCode::OK, Json(LoginResponse { tokens: token_pair })))
}

pub fn router() -> Router<AppState> {
  Router::new()
    .route("/users", get(get_all_users))
    .route("/users/name/:name", get(find_user))
    .route("/auth/register", post(add_user))
    // .route("/auth/login", post(login_user))
    .route("/auth/test", get(test_user_authorized).layer(middleware::from_fn(logged_in_guard)))
    .route("/auth/refresh/:refresh_token", get(refresh_user_token))
}