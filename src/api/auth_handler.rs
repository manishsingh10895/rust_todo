use actix_web::{route, web, HttpResponse};
use diesel::prelude::*;

use crate::{
    api::{
        auth_utils::{encode_token, hash_password, verify_hash},
        errors::AuthError,
        middlewares::auth::Claims,
    },
    models::{
        todo_model::Todo,
        user_model::{SlimUser, User},
        Pool,
    },
};

use super::{
    dtos::auth::{LoginDTO, LoginResponseDTO, SignupRequestDTO, SignupResponseDTO},
    errors::TodoApiError,
};

#[route("/auth/login", method = "POST")]
/// Login a user
pub async fn login(
    request_data: web::Json<LoginDTO>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = web::block(move || get_user(pool, request_data.into_inner())).await??;

    Ok(HttpResponse::Ok().json(&user))
}

#[route("/auth/signup", method = "POST")]
pub async fn signup(
    request_data: web::Json<SignupRequestDTO>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user = web::block(move || insert_new_user(pool, request_data.into_inner())).await??;

    Ok(HttpResponse::Ok().json(&user))
}

/// Get User by email
fn get_user(pool: web::Data<Pool>, user_data: LoginDTO) -> Result<LoginResponseDTO, TodoApiError> {
    use crate::schema::users::dsl::*;

    let conn = &pool.get()?;

    let found_user: Result<SlimUser, _> = users
        .filter(email.eq(user_data.email))
        .load::<User>(conn)
        .map_err(|_db_error| {
            eprintln!("Db error {}", _db_error);
            TodoApiError::InternalServerError
        })
        .and_then(|mut result| {
            if let Some(user) = result.pop() {
                if verify_hash(&user.password, &user_data.password)? {
                    Ok(user.into())
                } else {
                    Err(TodoApiError::AuthError(AuthError::Unauthorized))
                }
            } else {
                Err(TodoApiError::BadRequest("No user found".into()))
            }
        });

    match found_user {
        Ok(user) => {
            let token = encode_token::<Claims>(&user).unwrap();

            Ok(LoginResponseDTO {
                id: user.id.to_string(),
                email: user.email,
                token,
            })
        }
        Err(err) => Err(err),
    }
}

/// Query Database to insert a new user on signup
fn insert_new_user(
    pool: web::Data<Pool>,
    user_data: SignupRequestDTO,
) -> Result<SignupResponseDTO, TodoApiError> {
    use crate::schema::users::dsl::email;
    use crate::schema::users::dsl::*;

    let conn = &pool.get()?;

    let user: Result<SlimUser, _> = users
        .filter(email.eq(&user_data.email))
        .load::<User>(conn)
        .map_err(|_db_error| {
            eprintln!("Db Error User {}", _db_error);
            TodoApiError::InternalServerError
        })
        .and_then(|mut result| {
            if let Some(_) = result.pop() {
                return Err(TodoApiError::BadRequest("User Already Exists".into()));
            } else {
                let hashed = hash_password(&user_data.password).unwrap();

                let new_user = User::from_details(user_data.name, user_data.email, hashed.into());
                let _: User = diesel::insert_into(users)
                    .values(&new_user)
                    .get_result(conn)?;

                Ok(new_user.into())
            }
        });

    match user {
        Ok(slim_user) => {
            let token = encode_token::<Claims>(&slim_user).unwrap();

            return Ok(SignupResponseDTO {
                email: slim_user.email,
                id: slim_user.id.to_string(),
                token,
            });
        }
        Err(err) => {
            return Err(err);
        }
    }
}
