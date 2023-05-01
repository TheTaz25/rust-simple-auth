use axum::{
    Router,
    extract::State,
    extract::Path,
    extract::Json,
    routing::get,
    http::StatusCode,
};
use serde::{Serialize};
use std::{sync::{Mutex, Arc}};
use dotenv::dotenv;

#[derive(Clone)]
struct AppState {
    current_count: Arc<Mutex<i32>>,
    user_list: Arc<Mutex<Vec<User>>>,
}

#[derive(Clone, Serialize)]
struct User {
    username: String,
    password: Option<String>,
}

#[derive(Serialize)]
struct UserListResponse {
    users: Vec<User>
}

impl User {
    // fn new_existing(username: String, password: String) -> Self {
    //     User {
    //         username,
    //         password: Some(password),
    //     }
    // }

    fn new(username: String) -> Self {
        User {
            username,
            password: None
        }
    }

    fn set_password(&mut self, password: String) {
        let hash_result = bcrypt::hash(password, 10);
        match hash_result {
            Ok(result) => self.password = Some(result),
            Err(e) => panic!("Couldn't hash password!\n{}", e)
        }
    }
}

async fn get_state  (
    State(state): State<AppState>,
) -> (StatusCode, String) {
    let count = state.current_count.lock().unwrap();
    (StatusCode::OK, count.to_string())
}

async fn add_to_state (
    State(state): State<AppState>,
    Path(value): Path<i32>,
) -> (StatusCode, String) {
    let mut count = state.current_count.lock().unwrap();
    *count += value;
    (StatusCode::OK, ("Added ".to_string() + &value.to_string() + &" to count!".to_string()))
}

fn get_default_admin_user () -> Vec<User> {
    let first_admin_user = std::env::var("ADMIN_USER").expect("ADMIN_USER needs to be set!");
    let first_admin_pass = std::env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD needs to be set!");
    let mut new_user = User::new(first_admin_user);
    new_user.set_password(first_admin_pass);
    vec![new_user]
}

// async fn add_user (
//     State(state): State<AppState>,
//     Json(body): Json<User>,
// ) -> StatusCode {
//     StatusCode::CREATED
// }

async fn get_all_users (
    State(state): State<AppState>
) -> (StatusCode, Json<UserListResponse>) {
    let users = state.user_list.lock().unwrap();
    let response = UserListResponse {
        users: users.to_vec()
    };
    (StatusCode::OK, Json(response))
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let state = AppState {
        current_count: Arc::new(Mutex::new(0)),
        user_list: Arc::new(Mutex::new(get_default_admin_user()))
    };

    let app = Router::new()
        .route("/", get(get_state))
        .route("/add/:value", get(add_to_state))
        .route("/users", get(get_all_users))
        .with_state(state);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
