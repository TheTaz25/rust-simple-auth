-- Your SQL goes here
CREATE TABLE users (
  user_id UUID NOT NULL PRIMARY KEY,
  username varchar(64) NOT NULL UNIQUE,
  password varchar(64) NOT NULL,
  admin bool DEFAULT false
)