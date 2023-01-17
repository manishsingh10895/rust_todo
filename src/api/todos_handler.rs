use std::cmp::Ordering;

use actix_web::{web, HttpResponse};
use serde_json::json;

use super::errors::TodoApiError;
use super::middlewares::auth::Authenticated;
use crate::api::auth_utils::verify_todo_owner;
use crate::api::dtos::todo::CreateTodoDTO;
use crate::models::todo_model::Todo;
use crate::models::Pool;

use diesel::prelude::*;
/// Create a new todo

pub async fn create_todo(
    request_data: web::Json<CreateTodoDTO>,
    pool: web::Data<Pool>,
    auth: Authenticated,
) -> Result<HttpResponse, actix_web::Error> {
    let inserted =
        web::block(move || insert_new_todo(pool, request_data.into_inner(), &auth.id)).await??;

    Ok(HttpResponse::Ok().json(&inserted))
}

/// Api handler for getting all todos for a user
pub async fn get_todos(
    auth: Authenticated,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut list = web::block(move || get_all_todos_for_user(pool, &auth.id)).await??;

    list.sort_by(|a, b| {
        if a.completed {
            return Ordering::Less;
        }

        if a.completed == b.completed {
            return Ordering::Equal;
        }

        if b.completed {
            return Ordering::Greater;
        }

        Ordering::Equal
    });

    Ok(HttpResponse::Ok().json(json!({ "todos": list })))
}

/// Api to Delete a TODO
pub async fn delete_todo(
    auth: Authenticated,
    params: web::Path<String>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    //TODO: Implement check for user before deleting
    let _ = web::block(move || remove_todo(pool, params.into_inner().as_str(), &auth.id)).await??;

    Ok(HttpResponse::Ok().finish())
}

/// Update a Todo's completeness
pub async fn mark_todo_as_complete(
    _: Authenticated, // Not checking for valid user, (YET)
    todo_id: web::Path<String>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let _ = web::block(move || update_todo_completeness(pool, todo_id.into_inner().as_str(), true))
        .await??;

    Ok(HttpResponse::Ok().finish())
}

/// Mark a todo as incomplete
pub async fn mark_todo_as_incomplete(
    _: Authenticated, // Not checking for valid user (YET)
    todo_id: web::Path<String>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let _ =
        web::block(move || update_todo_completeness(pool, todo_id.into_inner().as_str(), false))
            .await??;

    Ok(HttpResponse::Ok().finish())
}

/// Remove a todo
fn remove_todo(
    pool: web::Data<Pool>,
    todo_id: &str,
    requester_id: &str,
) -> Result<(), TodoApiError> {
    use crate::schema::todos::dsl::*;

    let conn = &pool.get()?;

    match uuid::Uuid::parse_str(todo_id) {
        Ok(_uid) => {
            // check if requester is the owner of the todo
            verify_todo_owner(conn, requester_id, todo_id)?;

            let todo_id = uuid::Uuid::parse_str(todo_id).unwrap();

            let update_count = diesel::delete(todos.filter(id.eq(todo_id))).execute(conn)?;

            if update_count > 0 {
                return Ok(());
            } else {
                return Err(TodoApiError::NotFound(String::from("Todo")));
            }
        }
        Err(e) => {
            eprintln!("Error while deleting Todo {}", e);
            Err(TodoApiError::BadRequest(String::from("Invalid Todo Id")))
        }
    }
}

fn insert_new_todo(
    pool: web::Data<Pool>,
    todo: CreateTodoDTO,
    requester_id: &str,
) -> Result<Todo, TodoApiError> {
    use crate::schema::todos::dsl::*;

    let conn = &pool.get()?;

    let new_todo = Todo::from(todo.title, uuid::Uuid::parse_str(requester_id).unwrap());

    let inserted = diesel::insert_into(todos)
        .values(&new_todo)
        .get_result(conn);

    match inserted {
        Ok(todo) => Ok(todo),
        Err(e) => {
            eprintln!("Error {}", e);
            Err(TodoApiError::InternalServerError)
        }
    }
}

/// Complete or Uncomplete a todo
fn update_todo_completeness(
    pool: web::Data<Pool>,
    todo_id: &str,
    is_complete: bool,
) -> Result<(), TodoApiError> {
    use crate::schema::todos::dsl::*;
    let conn = &pool.get()?;

    match uuid::Uuid::parse_str(todo_id) {
        Ok(uid) => {
            let _: usize = diesel::update(todos)
                .filter(id.eq(uid))
                .set(completed.eq(is_complete))
                .execute(conn)?;

            Ok(())
        }
        Err(e) => {
            eprintln!("Error while updating");
            eprintln!("{}", e);
            return Err(TodoApiError::BadRequest(String::from("Invalid Todo Id")));
        }
    }
}

/// Get all todos for a user
fn get_all_todos_for_user(
    pool: web::Data<Pool>,
    requester_id: &str,
) -> Result<Vec<Todo>, TodoApiError> {
    use crate::schema::todos::dsl::*;
    let conn = &pool.get()?;

    let todos_list = todos
        .filter(user_id.eq(uuid::Uuid::parse_str(requester_id).unwrap()))
        .load::<Todo>(conn)?;

    Ok(todos_list)
}

#[cfg(test)]
mod test {
    fn test_remove_todo() {}
}
