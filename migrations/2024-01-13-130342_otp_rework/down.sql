-- This file should undo anything in `up.sql`
ALTER TABLE otp DROP COLUMN user;
ALTER TABLE otp ADD COLUMN usages_left SMALLINT default 0;