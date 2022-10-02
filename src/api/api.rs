use actix_web::{self, web, App, HttpServer};
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;

use crate::models;

use super::{auth_handler, middlewares::auth::BasicAuth, todos_handler};

#[actix_web::main]
pub async fn start_server() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    std::env::set_var(
        "RUST_LOG",
        "simeple-authe_server=debug,actix_web=info,actix_server=info",
    );

    env_logger::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let api_url = std::env::var("API_URL").unwrap_or(String::from("localhost:9000"));

    let manager = ConnectionManager::<diesel::PgConnection>::new(database_url);

    let pool: models::Pool = Pool::builder()
        .build(manager)
        .expect("Failed to connection to PG database");

    let _: String = std::env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string());

    HttpServer::new(move || {
        App::new().app_data(web::Data::new(pool.clone())).service(
            web::scope("/api")
                .service(auth_handler::signup)
                .service(auth_handler::login)
                .service(
                    web::scope("/todo")
                        .wrap(BasicAuth)
                        .route("", web::get().to(todos_handler::get_todos))
                        .route("", web::post().to(todos_handler::create_todo))
                        .route("/{id}", web::delete().to(todos_handler::delete_todo))
                        .route(
                            "{id}/complete",
                            web::put().to(todos_handler::mark_todo_as_complete),
                        )
                        .route(
                            "{id}/incomplete",
                            web::put().to(todos_handler::mark_todo_as_incomplete),
                        ),
                ),
        )
    })
    .workers(1) // Num of threads
    .bind(api_url.as_str())?
    .run()
    .await
}
