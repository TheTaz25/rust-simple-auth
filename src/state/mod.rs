use std::sync::Arc;

use redis::Client;

use self::postgres_wrapper::WrappedPostgres;
use self::redis_wrapper::WrappedRedis;

type RedisClient = Client;

pub mod redis_wrapper;
pub mod postgres_wrapper;

#[derive(Clone)]
pub struct AppState {
  pub pool: Arc<WrappedPostgres>,
  pub redis: Arc<WrappedRedis>,
}