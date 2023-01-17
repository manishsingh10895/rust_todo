-- Your SQL goes here

ALTER TABLE todos ADD COLUMN user_id UUID NOT NULL;

ALTER TABLE todos ADD CONSTRAINT 
    user_todo_foriegn_key 
    FOREIGN KEY(user_id) REFERENCES users(id);
