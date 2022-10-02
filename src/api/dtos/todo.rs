use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTodoDTO {
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CompleteTodoDTO {
    pub id: String,
}
