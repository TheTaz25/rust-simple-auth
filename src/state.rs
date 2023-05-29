use std::sync::{Arc, Mutex};

use crate::PgPool;
use crate::api::auth::session::TokenList;

#[derive(Clone)]
pub struct AppState {
  pub token_list: Arc<Mutex<TokenList>>,
  pub pool: Arc<PgPool>,
}