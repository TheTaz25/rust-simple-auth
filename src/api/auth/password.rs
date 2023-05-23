

pub fn hash_password(password: String) -> Result<String, String> {
  let hash_result = bcrypt::hash(password, 10);
  match hash_result {
    Ok(result) => Ok(result),
    Err(_) => Err("Could not generate hash from password".to_string())
  }
}