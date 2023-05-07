use std::ops::Add;

use axum::http::StatusCode;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{prelude::Local, Duration};

#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct Token {
  token: Uuid,
  expires_at: i64,
}

impl Token {
  pub fn new(duration: Duration) -> Self {
    Token {
      token: Uuid::new_v4(),
      expires_at: Local::now().add(duration).timestamp_millis()
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
pub struct TokenPair {
  user: Uuid,
  access_token: Token,
  refresh_token: Token,
}


impl TokenPair {
  pub fn new(user: Uuid) -> Self {
    TokenPair {
      user,
      access_token: Token::new(Duration::days(14)),
      refresh_token: Token::new(Duration::days(31))
    }
  }
}

pub struct TokenList {
  pub list: Vec<TokenPair>
}

impl TokenList {
  pub fn new() -> Self {
    TokenList { list: vec![] }
  }

  pub fn add(&mut self, token: TokenPair) {
    self.list.push(token)
  }

  pub fn remove_by_access_token(&mut self, token_id: Uuid) {
    self.list.retain(|t| t.access_token.r#match(token_id))
  }

  pub fn remove_by_refresh_token(&mut self, token_id: Uuid) {
    self.list.retain(|t| t.refresh_token.r#match(token_id))
  }

  pub fn clear_all_invalid(&mut self) {
    self.list.retain(|t| t.refresh_token.expired())
  }

  pub fn access_token_valid(&self, token_id: Uuid) -> Result<(), StatusCode> {
    let found = self.list.iter().find(|t| t.access_token.r#match(token_id));

    match found {
      Some(pair) => if !pair.access_token.expired() { Ok(()) } else { Err(StatusCode::FORBIDDEN) },
      None => Err(StatusCode::FORBIDDEN)
    }
  }

  pub fn refresh_token_valid(&self, token_id: Uuid) -> bool {
    let found = self.list.iter().find(|t| t.refresh_token.r#match(token_id));
    match found {
      Some(pair) => !pair.refresh_token.expired(),
      None => false
    }
  }
}
