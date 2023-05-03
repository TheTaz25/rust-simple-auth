use serde::Serialize;

#[derive(Clone, Serialize)]
pub struct User {
  username: String,
  password: String,
  admin: bool,
}

fn hash_password(password: String) -> Result<String, &'static str> {
  let hash_result = bcrypt::hash(password, 10);
  match hash_result.ok().as_ref() {
    Some(result) => Ok(result.clone()),
    None => Err("Could not generate hash from password")
  }
}

pub fn get_default_admin_user () -> User {
  let admin_username = std::env::var("ADMIN_USER").expect("ADMIN_USER needs to be a string!");
  let admin_password = std::env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD needs to be a string!");
  User::new(admin_username, admin_password, true)
}

impl User {
  pub fn new(username: String, clear_text_password: String, admin: bool) -> Self {
    User {
      username,
      password: hash_password(clear_text_password).expect("Was not able to generate a hashed password"),
      admin,
    }
  }
  pub fn from_existing(username: String, hashed_password: String, admin: bool) -> Self {
    User {
      username,
      password: hashed_password,
      admin,
    }
  }
}