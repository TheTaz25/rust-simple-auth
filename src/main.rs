use std::sync::Arc;
use rust_auth::api::system_setup::init_admin_user::setup;
use rust_auth::state::postgres_wrapper::WrappedPostgres;
use dotenv::dotenv;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_http::trace::TraceLayer;
use tower_http::cors::CorsLayer;

use rust_auth::api::auth::auth::router as auth_router;
use rust_auth::api::user::user::router as user_router;
use rust_auth::api::otp::otp::router as otp_router;

use rust_auth::state::AppState;
use rust_auth::state::redis_wrapper::WrappedRedis;

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
        pool: Arc::new(pg_client),
        redis: Arc::new(redis_client),
    };

    let routes = auth_router(state.clone())
        .merge(user_router(state.clone()))
        .merge(otp_router(state.clone()))
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // run it with hyper on localhost:8080
    let listener = tokio::net::TcpListener::bind(&"0.0.0.0:8080").await.unwrap();

        axum::serve(listener, routes.into_make_service())
        .await
        .unwrap();
}
