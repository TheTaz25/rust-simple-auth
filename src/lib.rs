use bb8::PooledConnection;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};

pub mod api;

pub mod state;

pub mod utils;

pub mod middleware;

pub mod schema;

pub mod models;

pub type Pool<'a> = PooledConnection<'a, AsyncDieselConnectionManager<AsyncPgConnection>>;