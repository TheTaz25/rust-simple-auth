use std::sync::{Mutex, Arc};
use dotenv::dotenv;

use back_end_paper_2::api::auth::user::{self, router};
use back_end_paper_2::state::AppState;

// async fn add_user (
//     State(state): State<AppState>,
//     Json(body): Json<User>,
// ) -> StatusCode {
//     StatusCode::CREATED
// }

#[tokio::main]
async fn main() {
    dotenv().ok();

    let mut new_user_list = user::UserList::new();
    new_user_list.add(user::get_default_admin_user());

    let state = AppState {
        user_list: Arc::new(Mutex::new(new_user_list))
    };

    let routes = router().with_state(state);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
        .serve(routes.into_make_service())
        .await
        .unwrap();
}
