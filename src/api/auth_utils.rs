use actix_web::http::header::HeaderValue;
use argon2::Config;
use diesel::{r2d2::ConnectionManager, PgConnection};
use jsonwebtoken::{decode, decode_header, encode, DecodingKey, EncodingKey, Header, Validation};
use r2d2::PooledConnection;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_value, Value};

use crate::models::{todo_model::Todo, user_model::SlimUser};

use super::{
    errors::{AuthError, TodoApiError},
    middlewares::auth::Claims,
};
use diesel::prelude::*;

lazy_static::lazy_static! {
    pub static ref SECRET_KEY: String = std::env::var("SECRET_KEY").unwrap_or_else(|_| "0123".repeat(8));
}

const SALT: &'static [u8] = b"supersecuresalt";

const JWT_SECRET_KEY: &'static [u8] = b"secure jwt secret";

// Hashing

/// Hash a password, to secure
pub fn hash_password(password: &str) -> Result<String, TodoApiError> {
    let config = Config {
        secret: SECRET_KEY.as_bytes(),
        ..Default::default()
    };

    argon2::hash_encoded(password.as_bytes(), &SALT, &config).map_err(|err| {
        dbg!(err);
        TodoApiError::InternalServerError
    })
}

/// Verify password and hash are equal
pub fn verify_hash(hash: &str, password: &str) -> Result<bool, TodoApiError> {
    argon2::verify_encoded_ext(hash, password.as_bytes(), SECRET_KEY.as_bytes(), &[]).map_err(
        |err| {
            dbg!(err);
            TodoApiError::AuthError(crate::api::errors::AuthError::InvalidToken)
        },
    )
}

///Verifies that a user with `user_id` has access to todo with `todo_id`
pub fn verify_todo_owner(
    conn: &PooledConnection<ConnectionManager<PgConnection>>,
    requester_id: &str,
    todo_id: &str,
) -> Result<(), TodoApiError> {
    use crate::schema::todos::dsl::*;

    let todo_id = uuid::Uuid::parse_str(todo_id).unwrap();

    let requester_id = uuid::Uuid::parse_str(requester_id).unwrap();

    let result: Vec<Todo> = todos
        .filter(id.eq(todo_id))
        .filter(user_id.eq(requester_id))
        .load::<Todo>(conn)?;

    if result.len() == 0 {
        return Err(TodoApiError::NotFound("Todo".to_string()));
    }

    Ok(())
}

// JWT STUFF
pub trait Claimable<'a>: Serialize + From<&'a SlimUser> + DeserializeOwned {}

/// Create a jwt token for user
pub fn _encode_token(user: &SlimUser) -> Result<String, AuthError> {
    Ok(encode::<Claims>(
        &Header::new(jsonwebtoken::Algorithm::HS256),
        &user.into(),
        &EncodingKey::from_secret(JWT_SECRET_KEY),
    )?)
}

pub fn encode_token<'a, T: Claimable<'a>>(user: &'a SlimUser) -> Result<String, AuthError> {
    Ok(encode::<T>(
        &Header::new(jsonwebtoken::Algorithm::HS256),
        &user.into(),
        &EncodingKey::from_secret(JWT_SECRET_KEY),
    )?)
}

pub fn decode_token<'a, T: Claimable<'a>>(auth_header: &HeaderValue) -> Result<T, AuthError> {
    match auth_header.to_str() {
        Ok(auth_header_string) => {
            let token = auth_header_string.trim_start_matches("Bearer ");

            match decode_header(token) {
                Ok(jwt_header) => {
                    match decode::<Value>(
                        token,
                        &DecodingKey::from_secret(JWT_SECRET_KEY),
                        &Validation::new(jwt_header.alg),
                    ) {
                        Ok(raw_token) => {
                            let decoded = from_value::<T>(raw_token.claims.clone());

                            match decoded {
                                Ok(claims) => Ok(claims),
                                Err(e) => Err(AuthError::Claims(e)),
                            }
                        }

                        Err(_) => Err(AuthError::InvalidToken),
                    }
                }
                Err(e) => {
                    dbg!(e);
                    Err(AuthError::InvalidToken)
                }
            }
        }
        Err(_) => Err(AuthError::InvalidAuthorizationHeader),
    }
}

/// Verifies a jwt token
pub fn _decode_token(auth_header: &HeaderValue) -> Result<Claims, AuthError> {
    match auth_header.to_str() {
        Ok(auth_header_string) => {
            let token = auth_header_string.trim_start_matches("Bearer ");

            match decode_header(token) {
                Ok(jwt_header) => {
                    match decode::<Value>(
                        token,
                        &DecodingKey::from_secret(JWT_SECRET_KEY),
                        &Validation::new(jwt_header.alg),
                    ) {
                        Ok(raw_token) => {
                            let decoded = from_value::<Claims>(raw_token.claims.clone());

                            match decoded {
                                Ok(claims) => Ok(claims),
                                Err(e) => Err(AuthError::Claims(e)),
                            }
                        }

                        Err(_) => Err(AuthError::InvalidToken),
                    }
                }
                Err(e) => {
                    dbg!(e);
                    Err(AuthError::InvalidToken)
                }
            }
        }
        Err(_) => Err(AuthError::InvalidAuthorizationHeader),
    }
}
