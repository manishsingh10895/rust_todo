use super::{super::schema::*, Pool};
use crate::api::errors::TodoApiError;
use actix_web::web;
use diesel::prelude::*;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use uuid;

#[derive(Debug, Deserialize, Serialize, Insertable, Queryable)]
#[table_name = "users"]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub password: String,
    pub name: String,
}

impl User {
    pub fn from_details<T: Into<String>>(name: T, email: T, password: T) -> User {
        User {
            email: email.into(),
            password: password.into(),
            id: uuid::Uuid::new_v4(),
            name: name.into(),
            created_at: chrono::Local::now().naive_local(),
            updated_at: chrono::Local::now().naive_local(),
        }
    }
}

pub struct SlimUser {
    pub email: String,
    pub id: uuid::Uuid,
    pub name: String,
}

impl AsRef<SlimUser> for SlimUser {
    fn as_ref(&self) -> &SlimUser {
        self
    }
}

impl From<User> for SlimUser {
    fn from(user: User) -> Self {
        Self {
            name: user.name,
            email: user.email,
            id: user.id,
        }
    }
}

/// Gets a user by id
pub fn get_user_by_id(pool: web::Data<Pool>, user_id: String) -> Result<User, TodoApiError> {
    use crate::schema::users::dsl::*;

    let conn = &pool.get()?;

    let uid: uuid::Uuid = uuid::Uuid::parse_str(user_id.as_str()).unwrap();

    let user: Result<User, _> = users
        .filter(id.eq(uid))
        .load::<User>(conn)
        .map_err(|db_error| {
            eprintln!("Db error {}", db_error);
            TodoApiError::InternalServerError
        })
        .and_then(|mut result| {
            if let Some(user) = result.pop() {
                Ok(user)
            } else {
                Err(TodoApiError::NotFound("User".to_string()))
            }
        });

    user
}
