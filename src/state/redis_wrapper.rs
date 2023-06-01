use axum::http::StatusCode;
use redis::aio::MultiplexedConnection;

use super::RedisClient;

#[derive(Clone)]
pub struct WrappedRedis {
  redis: RedisClient
}

impl WrappedRedis {
  pub fn new() -> Self {
    let client = redis::Client::open("redis://127.0.0.1").unwrap();
    WrappedRedis {
      redis: client
    }
  }

  pub async fn get_connection(&self) -> Result<MultiplexedConnection, StatusCode> {
    self.redis.get_multiplexed_tokio_connection().await.or_else(|_| Err(StatusCode::INTERNAL_SERVER_ERROR))
  }
}