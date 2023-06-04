use std::sync::{Mutex, Arc};
use back_end_paper_2::api::system_setup::init_admin_user::setup;
use back_end_paper_2::state::postgres_wrapper::WrappedPostgres;
use dotenv::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_http::trace::TraceLayer;

use back_end_paper_2::api::auth::user::router;
use back_end_paper_2::state::AppState;
use back_end_paper_2::api::auth::session::TokenList;
use back_end_paper_2::state::redis_wrapper::WrappedRedis;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // BEGIN TRACING SETUP
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| "tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    // END TRACING SETUP

    // BEGIN Database Setup
    let pg_client = WrappedPostgres::new().await;
    
    let adm_setup_result = setup(&pg_client.postgres).await;

    match adm_setup_result {
        Ok(_) => println!("Fresh start. Initialized the provided adm-default user"),
        Err(_) => println!("An admin user did already exist, skipped setup of adm user")
    }
    // END Database Setup
    // BEGIN REDIS SETUP
    let redis_client = WrappedRedis::new();
    // END REDIS SETUP


    let state = AppState {
        token_list: Arc::new(Mutex::new(TokenList::new())),
        pool: Arc::new(pg_client),
        redis: Arc::new(redis_client),
    };

    let routes = router(state.clone())
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    // run it with hyper on localhost:3000
    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(routes.into_make_service())
        .await
        .unwrap();
}
