use std::io::Error;

use api::api::start_server;

#[macro_use]
extern crate diesel;

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use utils::is_server_running;

use crate::utils::{make_api_url, save_token};
mod api;
mod config;
mod errors;
mod models;
mod schema;
mod todo_commands;
mod ui;
mod utils;

#[derive(Deserialize, Serialize)]
struct TodoConfig {
    api_url: String,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Login,
    Signup,
    #[clap(alias = "ls")]
    List,
    #[clap(alias = "c")]
    Create,
}

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = "Manage todos from command line")]
struct TodoArgs {
    #[clap(short = 's', long = "start-server")]
    start_server: bool,

    #[clap(subcommand)]
    command: Option<Commands>,
}
// Signup a user through cli
fn prompt_signup() -> Result<(), Box<dyn std::error::Error>> {
    use inquire::{Password, Text};
    use reqwest::header::CONTENT_TYPE;

    println!("Signup to Todo");

    let name = Text::new("Name").prompt()?;

    let email = Text::new("Email").prompt()?;

    let pass = Password::new("Password").prompt()?;

    let client = reqwest::blocking::Client::new();

    let resp = client
        .post(make_api_url("auth/signup"))
        .header(CONTENT_TYPE, "application/json")
        .json::<serde_json::Value>(
            &serde_json::json!({"email": email, "password": pass, "name": name}),
        )
        .send()?;

    let res_status = resp.status();
    let resp_json: serde_json::Value = resp.json()?;

    if resp_json.is_object() && res_status == 200 {
        let data = resp_json.as_object().ok_or("Invalid response")?;

        let token = data.get("token").ok_or("Token Not Found, Signup Failed")?;

        save_token(token.as_str().unwrap())?;

        println!("Signup Successful");

        println!("You are now logged in");

        return Ok(());
    }

    Err(Box::new(Error::new(
        std::io::ErrorKind::Other,
        "Invalid Response for Signup",
    )))
}

// Prompt login
fn prompt_login() -> Result<(), Box<dyn std::error::Error>> {
    use inquire::{Password, Text};
    use reqwest::header::CONTENT_TYPE;

    println!("Login to Todo");

    let email = Text::with_default(Text::new("Email"), "manishsingh10895@gmail.com")
        .with_help_message("Enter Email")
        .prompt()?;

    let pass = Password::new("Password").prompt()?;

    let client = reqwest::blocking::Client::new();

    let resp = client
        .post(make_api_url("auth/login"))
        .header(CONTENT_TYPE, "application/json")
        .json::<serde_json::Value>(&serde_json::json!({"email": email, "password": pass}))
        .send()?;

    let resp_json: serde_json::Value = resp.json()?;

    if resp_json.is_object() {
        let data = resp_json.as_object().ok_or("Invalid response")?;

        let token = data
            .get("token")
            .ok_or("Token Not Found in response, Login Failed")?;

        save_token(token.as_str().unwrap())?;

        println!("You are not logged in");

        return Ok(());
    }

    Err(Box::new(Error::new(
        std::io::ErrorKind::Other,
        "Invalid Response for Login",
    )))
}

/// Wrapper function for looping a prompt function
/// if error occurs
fn super_prompt(title: &str, function: Box<dyn Fn() -> Result<(), Box<dyn std::error::Error>>>) {
    loop {
        println!("\n{}\n", title);
        let res = function();

        if res.is_ok() {
            break;
        } else {
            eprintln!("{:?}", res.err());

            let response = inquire::Confirm::new("Continue")
                .with_default(true)
                .prompt();

            if let Ok(to_continue) = response {
                if to_continue {
                    continue;
                }
            }

            break;
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = std::env::args();

    let args = TodoArgs::parse_from(args);

    if args.start_server {
        if !is_server_running("5900").unwrap() {
            println!("Starting Server on PORT 5900");
            start_server()?;
        } else {
            panic!("Server Already Running");
        }
    }

    match &args.command {
        Some(Commands::Login) => {
            super_prompt("Login", Box::new(prompt_login));
        }
        Some(Commands::Signup) => {
            super_prompt("Signup", Box::new(prompt_signup));
        }
        Some(Commands::Create) => {
            let x = todo_commands::create_new_todo();

            match x {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        }
        Some(Commands::List) => {
            let res = todo_commands::list_todos();

            match res {
                Err(e) => {
                    eprintln!("{}", e);
                }
                _ => {}
            }
        }
        None => {}
    }

    // is_server_running("9000");

    Ok(())
}
