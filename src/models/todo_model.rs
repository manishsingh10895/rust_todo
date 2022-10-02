use crate::schema::*;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, Queryable)]
#[table_name = "todos"]
pub struct Todo {
    pub id: uuid::Uuid,
    pub title: String,
    pub completed: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub user_id: uuid::Uuid,
}

impl Todo {
    pub fn from_title(title: String, user_id: uuid::Uuid) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            completed: false,
            title,
            user_id,
            created_at: chrono::Local::now().naive_local(),
            updated_at: chrono::Local::now().naive_local(),
        }
    }
}
