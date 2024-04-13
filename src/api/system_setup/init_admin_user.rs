use diesel_async::RunQueryDsl;

use crate::{models::user::NewAdmUser, api::auth::password::hash_password, PgPool};

fn get_default_admin_user () -> NewAdmUser {
  let username = std::env::var("ADMIN_USER").expect("ADMIN_USER environment config should be set!");
  let password = std::env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD environment config should be set!");

  NewAdmUser {
    user_id: uuid::Uuid::new_v4(),
    username,
    password: hash_password(password).expect("Failed to hash a password!"),
    admin: Some(true)
  }
}

pub async fn setup(pool: &PgPool) -> Result<(), ()> {
  use crate::schema::users;

  let adm_user = get_default_admin_user();
  let mut connection = pool.get().await.unwrap();
  
  let inserted = diesel::insert_into(users::table)
  .values(adm_user)
  .on_conflict(users::username)
  .do_nothing()
  .execute(&mut connection)
  .await.or_else(|_| Err(()))?;

  match inserted {
    0 => Err(()),
    _ => Ok(())
  }
}