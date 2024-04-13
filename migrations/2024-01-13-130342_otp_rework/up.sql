-- Your SQL goes here
ALTER TABLE otp DROP COLUMN usages_left;
ALTER TABLE otp ADD COLUMN "user" UUID REFERENCES users (user_id);

