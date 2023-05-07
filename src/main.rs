use std::sync::{Mutex, Arc};
use dotenv::dotenv;

use back_end_paper_2::api::auth::user::{self, router};
use back_end_paper_2::state::AppState;
use back_end_paper_2::api::auth::session::TokenList;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let mut new_user_list = user::UserList::new();
    new_user_list.add(user::get_default_admin_user());

    let state = AppState {
        user_list: Arc::new(Mutex::new(new_user_list)),
        token_list: Arc::new(Mutex::new(TokenList::new())),
    };

    let routes = router().with_state(state);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(routes.into_make_service())
        .await
        .unwrap();
}
