use std::str::FromStr;

use inquire::Text;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

use crate::{
    models::todo_model::Todo,
    ui::todo_list_renderer::render_todo_list,
    utils::{get_saved_token, make_api_url},
};

/// Prompt user to create new todo
pub fn create_new_todo() -> Result<(), Box<dyn std::error::Error>> {
    let title = Text::new("Title")
        .with_help_message("Title for your new todo")
        .prompt()?;

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post(make_api_url("todo"))
        .header(CONTENT_TYPE, "application/json")
        .json::<serde_json::Value>(&serde_json::json!({ "title": title }))
        .send()?;

    let _: serde_json::Value = resp.json()?;

    Ok(())
}

/// List all the todos for user
pub fn list_todos() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();

    let token = get_saved_token()?;

    let response = client
        .get(make_api_url("todo"))
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .header(CONTENT_TYPE, "application/json")
        .send()?;

    match response.status() {
        reqwest::StatusCode::OK => {
            let list = response.text()?;

            let json = serde_json::Value::from_str(list.as_str())?;

            let todos = json.get("todos").unwrap().to_owned();

            let list: Vec<Todo> = serde_json::from_value(todos)?;

            render_todo_list(list)?;
        }
        reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
            eprintln!("Login First");
        }
        _ => {
            eprintln!("Main line error");
        }
    }

    Ok(())
}
