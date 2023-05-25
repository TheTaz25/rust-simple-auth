use std::sync::{Mutex, Arc};
use back_end_paper_2::api::system_setup::init_admin_user::setup;
use diesel_async::{pooled_connection::AsyncDieselConnectionManager};
use dotenv::dotenv;

use back_end_paper_2::api::auth::user::{self, router};
use back_end_paper_2::state::AppState;
use back_end_paper_2::api::auth::session::TokenList;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_url = build_db_from_env();
    let db_config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(db_url);
    let db_pool = bb8::Pool::builder().build(db_config).await.expect("Failed to setup a database pool");

    let adm_setup_result = setup(&db_pool).await;

    match adm_setup_result {
        Ok(_) => println!("Fresh start. Initialized the provided adm-default user"),
        Err(_) => println!("An admin user did already exist, skipped setup of adm user")
    }

    let new_user_list = user::UserList::new();

    let state = AppState {
        user_list: Arc::new(Mutex::new(new_user_list)),
        token_list: Arc::new(Mutex::new(TokenList::new())),
        pool: Arc::new(db_pool)
    };

    let routes = router()
        .with_state(state);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(routes.into_make_service())
        .await
        .unwrap();
}

fn build_db_from_env() -> String {
    let db_user = std::env::var("DB_USER").expect("env var 'DB_USER' should contain an existing database username");
    let db_pass = std::env::var("DB_PASS").expect("env var 'DB_PASS' should contain the password for 'DB_USER'");
    let db_host = std::env::var("DB_HOST").expect("env var 'DB_HOST' should be set to host running the database");
    let db_name = std::env::var("DB_NAME").expect("env var 'DB_NAME' should be set to the database that will be used");

    format!("postgres://{}:{}@{}/{}", db_user, db_pass, db_host, db_name)
}