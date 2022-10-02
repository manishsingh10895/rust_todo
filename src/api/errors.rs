use actix_web::{
    body::BoxBody,
    http::{
        self,
        header::{self, HeaderValue},
    },
    HttpResponse, ResponseError,
};
use derive_more::Display;
use diesel::result::{DatabaseErrorKind, Error as DBError};
use jsonwebtoken::errors::ErrorKind;
use serde_json::json;
use std::convert::From;
use uuid::Error as ParseError;

#[derive(Debug)]
pub enum AuthError {
    Claims(serde_json::Error),
    ///Token is invalid
    InvalidToken,
    NoAuthorizationHeader,
    InvalidAuthorizationHeader,
    TokenExpired,

    Unauthorized,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAuthorizationHeader => {
                write!(f, "Authorization header is not in valid format ")
            }
            Self::NoAuthorizationHeader => write!(f, "No Authorization Header"),
            Self::Claims(e) => write!(f, "Error while Deserializing JWT: {}", e),
            Self::InvalidToken => write!(f, "Invalid JWT Token"),
            Self::TokenExpired => write!(f, "Token Expired"),
            Self::Unauthorized => write!(f, "Unauthorized"),
        }
    }
}

#[derive(Debug, Display)]
pub enum TodoApiError {
    #[display(fmt = "Internal Server Error")]
    InternalServerError,

    #[display(fmt = "BadRequest: {}", _0)]
    BadRequest(String),

    #[display(fmt = "Database Connection Error")]
    DatabaseConnectionError,

    #[display(fmt = "Authentication Error :{}", _0)]
    AuthError(AuthError),

    #[display(fmt = "{} Not Found", _0)]
    NotFound(String),
}

impl TodoApiError {
    pub fn to_response(&self) -> HttpResponse {
        self.error_response()
    }
}

impl ResponseError for TodoApiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            TodoApiError::InternalServerError => http::StatusCode::INTERNAL_SERVER_ERROR,
            TodoApiError::AuthError(_) => http::StatusCode::UNAUTHORIZED,
            TodoApiError::BadRequest(_) => http::StatusCode::BAD_REQUEST,
            TodoApiError::NotFound(_) => http::StatusCode::NOT_FOUND,
            _ => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        let mut res = HttpResponse::new(self.status_code());

        res.headers_mut().append(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        res.set_body(BoxBody::new(json!({"error": self.to_string()}).to_string()))
    }
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.into_kind() {
            ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            _ => AuthError::InvalidToken,
        }
    }
}

impl From<ParseError> for TodoApiError {
    fn from(_: ParseError) -> Self {
        TodoApiError::BadRequest("Error while parsing uuid".to_string())
    }
}

impl From<r2d2::Error> for TodoApiError {
    fn from(_: r2d2::Error) -> Self {
        TodoApiError::DatabaseConnectionError
    }
}

impl From<jsonwebtoken::errors::Error> for TodoApiError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.into_kind() {
            ErrorKind::ExpiredSignature => TodoApiError::AuthError(AuthError::TokenExpired),
            _ => TodoApiError::InternalServerError,
        }
    }
}

impl From<DBError> for TodoApiError {
    fn from(error: DBError) -> Self {
        match error {
            DBError::DatabaseError(kind, info) => {
                if let DatabaseErrorKind::UniqueViolation = kind {
                    let message: String =
                        info.details().unwrap_or_else(|| info.message()).to_string();

                    return TodoApiError::BadRequest(message);
                }
                TodoApiError::InternalServerError
            }
            _ => TodoApiError::InternalServerError,
        }
    }
}
