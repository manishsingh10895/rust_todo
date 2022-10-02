-- This file should undo anything in `up.sql`

ALTER TABLE todos DROP CONSTRAINT user_todo_foriegn_key;

ALTER TABLE todos DROP COLUMN user_id;
