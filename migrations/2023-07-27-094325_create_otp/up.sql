-- Your SQL goes here
CREATE TYPE otp_type AS ENUM ('REGISTER', 'PW_RESET');

CREATE TABLE IF NOT EXISTS otp (
  id SERIAL PRIMARY KEY,
  code VARCHAR(6) NOT NULL UNIQUE,
  code_type otp_type NOT NULL,
  usages_left SMALLINT default 0
);
