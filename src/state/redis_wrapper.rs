use axum::http::StatusCode;
use redis::{aio::MultiplexedConnection, AsyncCommands, Cmd};
use uuid::Uuid;

use crate::api::auth::session::TokenPair;

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

  pub async fn save_token_pair_for_user(&self, pair: &TokenPair) -> Result<(), StatusCode> {
    let mut con = self.get_connection().await?;

    redis::pipe()
      .add_command(build_set_ex_cmd(
        format!("ACCESS:{}", pair.get_access_token_string()),
        format!("{}:{}", pair.get_id_string(), pair.get_refresh_token_string()),
        pair.access_token.duration,
      )).ignore()
      .add_command(build_set_ex_cmd(
        format!("REFRESH:{}", pair.get_refresh_token_string()),
        format!("{}:{}", pair.get_id_string(), pair.get_access_token_string()),
        pair.refresh_token.duration,
      ))
      .query_async(&mut con).await.or_else(|x| {
        println!("{}", x);
        Err(StatusCode::INTERNAL_SERVER_ERROR)
      })?;

    Ok(())
  }

  async fn get_user_for_token(&self, token: &String) -> Result<Uuid, StatusCode> {
    let mut con = self.get_connection().await?;

    let raw_user_id: String = con.get(token).await.or_else(|_| Err(StatusCode::UNAUTHORIZED))?;

    Uuid::parse_str(&raw_user_id).or_else(|_| Err(StatusCode::INTERNAL_SERVER_ERROR))
  }

  pub async fn get_user_for_access_token(&self, access_token: &String) -> Result<Uuid, StatusCode> {
    self.get_user_for_token(&format!("ACCESS:{}", access_token)).await
  }

  pub async fn get_user_from_refresh_token(&self, refresh_token: &String) -> Result<Uuid, StatusCode> {
    self.get_user_for_token(&format!("REFRESH:{}", refresh_token)).await
  }
}

fn build_set_ex_cmd(key: String, value: String, duration: i64) -> Cmd {
  Cmd::new()
    .arg("SET")
    .arg(key)
    .arg(value)
    .arg("EX")
    .arg(duration)
    .to_owned()
}