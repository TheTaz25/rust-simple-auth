use std::sync::{Arc, Mutex};

use redis::Client;

use crate::PgPool;
use crate::api::auth::session::TokenList;

type RedisClient = Client;

#[derive(Clone)]
pub struct AppState {
  pub token_list: Arc<Mutex<TokenList>>,
  pub pool: Arc<PgPool>,
  pub redis: Arc<RedisClient>,
}