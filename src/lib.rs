use bb8::Pool;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};

pub mod api;

pub mod state;

pub mod utils;

pub mod middleware;

pub mod schema;

pub mod models;

pub type PgPool = Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;
