use redis::{aio::MultiplexedConnection, AsyncCommands, Cmd};
use uuid::Uuid;

use crate::{api::auth::session::TokenPair, utils::error::Fault};

use super::RedisClient;

#[derive(Clone)]
pub struct WrappedRedis {
  redis: RedisClient
}

impl WrappedRedis {
  pub fn new() -> Self {
    let env_redis = std::env::var("REDIS_HOST").expect("env var 'REDIS_HOST' should contain name of redis host system");
    let redis_url = format!("redis://{}", env_redis);
    let client = redis::Client::open(redis_url).unwrap();
    WrappedRedis {
      redis: client
    }
  }

  pub async fn get_connection(&self) -> Result<MultiplexedConnection, Fault> {
    self.redis.get_multiplexed_tokio_connection().await.or_else(|_| Err(Fault::DatabaseConnection))
  }

  pub async fn save_token_pair_for_user(&self, pair: &TokenPair) -> Result<(), Fault> {
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
      .query_async(&mut con).await.or_else(|_| {
        Err(Fault::DatabaseConnection)
      })?;

    Ok(())
  }

  async fn get_user_for_token(&self, token: &String) -> Result<Uuid, Fault> {
    let mut con = self.get_connection().await?;

    let result: String = con.get(token).await.or_else(|_| Err(Fault::NotLoggedIn))?;

    let splits: Vec<&str> = result.split(":").into_iter().collect();

    let parsed = Uuid::parse_str(&splits[0]).or_else(|_| Err(Fault::UuidConversion))?;

    Ok(parsed)
  }

  pub async fn get_user_for_access_token(&self, access_token: &String) -> Result<Uuid, Fault> {
    self.get_user_for_token(&format!("ACCESS:{}", access_token)).await
  }

  pub async fn get_user_from_refresh_token(&self, refresh_token: &String) -> Result<Uuid, Fault> {
    self.get_user_for_token(&format!("REFRESH:{}", refresh_token)).await
  }

  pub async fn clear_token(&self, token: &String) -> Result<(), Fault> {
    let mut con = self.get_connection().await?;
    
    con.del(format!("ACCESS:{}", token)).await.or_else(|_| Err(Fault::Unexpected))?;

    Ok(())
  }

  pub async fn invalidate_refresh_token_and_get_result(&self, token: Uuid) -> Result<(String, String), Fault> {
    let mut con = self.get_connection().await?;

    let result: String = con.get_del(format!("REFRESH:{}", token.to_string())).await.or_else(|_| Err(Fault::NotLoggedIn))?;

    let splits: Vec<&str> = result.split(":").into_iter().collect();

    Ok((splits[0].to_string(), splits[1].to_string()))
  }

  pub async fn invalidate_session_by_access_token(&self, token: Uuid) -> Result<(), Fault> {
    let mut con = self.get_connection().await?;

    let result: String = con.get_del(format!("ACCESS:{}", token.to_string())).await.or_else(|_| Err(Fault::NotLoggedIn))?;

    let refresh_token: String = result.split(":").into_iter().last().unwrap().to_string();

    con.del(format!("REFRESH:{}", refresh_token)).await.or_else(|_| Err(Fault::Unexpected))?;

    Ok(())
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
