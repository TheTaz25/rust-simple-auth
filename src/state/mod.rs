use std::sync::{Arc, Mutex};

use redis::Client;

// use crate::PgPool;
use crate::api::auth::session::TokenList;

use self::postgres_wrapper::WrappedPostgres;
use self::redis_wrapper::WrappedRedis;

type RedisClient = Client;

pub mod redis_wrapper;
pub mod postgres_wrapper;

#[derive(Clone)]
pub struct AppState {
  pub token_list: Arc<Mutex<TokenList>>,
  pub pool: Arc<WrappedPostgres>,
  pub redis: Arc<WrappedRedis>,
}