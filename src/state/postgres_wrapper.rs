use std::{sync::Arc, future::Future};

use bb8::PooledConnection;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};

use crate::{PgPool, utils::error::Fault};

#[derive(Clone)]
pub struct WrappedPostgres {
  pub postgres: Arc<PgPool>
}

pub struct WrappedPooledConnection<'a> {
  pub connection: PooledConnection<'a, AsyncDieselConnectionManager<AsyncPgConnection>>
}

impl WrappedPostgres {
  pub async fn new() -> Self {
    let db_url = build_db_str_from_env();
    let db_config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(db_url);
    let pool = bb8::Pool::builder().build(db_config).await.expect("Failed to setup a database pool");
    WrappedPostgres { postgres: Arc::new(pool) }
  }

  pub async fn get_connection(&self) -> Result<Box<WrappedPooledConnection>, Fault> {
    self.postgres.get().await
      .or_else(|_| Err(Fault::DatabaseConnection))
      .and_then(|c| Ok(Box::new(WrappedPooledConnection { connection: c })))
  }

  // TODO: Need to do more research on lifetimes as this does not seem to work as I want it to
  #[allow(dead_code)]
  pub async fn with_connection<
    'a,
    F,
    Fut,
    U
  >(&self, executor: F) -> Result<U, Fault> where
    F: FnOnce(&mut Box<WrappedPooledConnection>) -> Fut,
    Fut: Future<Output = Result<U, Fault>>,
  {
    let mut connection = self.get_connection().await?;

    let res = executor(&mut connection).await?;

    Ok(res)
  }
}

fn build_db_str_from_env() -> String {
  let db_user = std::env::var("DB_USER").expect("env var 'DB_USER' should contain an existing database username");
  let db_pass = std::env::var("DB_PASS").expect("env var 'DB_PASS' should contain the password for 'DB_USER'");
  let db_host = std::env::var("DB_HOST").expect("env var 'DB_HOST' should be set to host running the database");
  let db_name = std::env::var("DB_NAME").expect("env var 'DB_NAME' should be set to the database that will be used");

  format!("postgres://{}:{}@{}/{}", db_user, db_pass, db_host, db_name)
}