use std::fmt::Display;

use crate::api::errors::TodoApiError;

#[derive(Debug)]
pub enum TodoError {
    #[allow(dead_code)]
    ApiError(TodoApiError),
    HttpErrror(String),
    OperationError,
}

pub type BaseError = Box<dyn std::error::Error>;

impl Display for TodoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ApiError(e) => {
                write!(f, "{}", e.to_string())
            }
            Self::HttpErrror(e) => {
                write!(f, "{}", e)
            }
            Self::OperationError => {
                write!(f, "Some error is the")
            }
        }
    }
}

impl From<reqwest::Error> for TodoError {
    fn from(e: reqwest::Error) -> Self {
        TodoError::HttpErrror(e.to_string())
    }
}

impl std::error::Error for TodoError {
    fn description(&self) -> &str {
        match self {
            TodoError::ApiError(_) => "Error Occured in Api",
            TodoError::HttpErrror(_) => "Error Occured in http request",
            TodoError::OperationError => "Some Error operation",
        }
    }
}

impl From<BaseError> for TodoError {
    fn from(_: BaseError) -> Self {
        TodoError::OperationError
    }
}
