use std::sync::{Arc, Mutex};

use crate::api::auth::user::UserList;

#[derive(Clone)]
pub struct AppState {
  pub user_list: Arc<Mutex<UserList>>
}