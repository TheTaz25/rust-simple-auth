use std::ops::Add;

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{prelude::Local, Duration};

#[derive(Serialize, Deserialize, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Token {
  pub token: Uuid,
  expires_at: i64,
  pub duration: i64,
}

impl Token {
  pub fn new(duration: Duration) -> Self {
    Token {
      token: Uuid::new_v4(),
      expires_at: Local::now().add(duration).timestamp_millis(),
      duration: duration.num_seconds()
    }
  }

  pub fn r#match(&self, token: Uuid) -> bool {
    self.token.to_string() == token.to_string()
  }

  pub fn expired(&self) -> bool {
    let now = Local::now().timestamp_millis();
    self.expires_at < now
  }
}

#[derive(Serialize, Deserialize, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenPair {
  pub user: Uuid,
  pub access_token: Token,
  pub refresh_token: Token,
}


impl TokenPair {
  pub fn new(user: &Uuid) -> Self {
    TokenPair {
      user: user.to_owned(),
      access_token: Token::new(Duration::days(14)),
      refresh_token: Token::new(Duration::days(31))
    }
  }

  pub fn get_id_string(&self) -> String {
    self.user.to_string()
  }

  pub fn get_access_token_string(&self) -> String {
    self.access_token.token.to_string()
  }

  pub fn get_refresh_token_string(&self) -> String {
    self.refresh_token.token.to_string()
  }
}
