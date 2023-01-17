use futures::{
    future::{ok, ready, LocalBoxFuture, Ready},
    FutureExt,
};

use actix_web::{
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    FromRequest, HttpMessage,
};
use chrono::{serde::ts_seconds, DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    api::{
        auth_utils::{decode_token, Claimable},
        errors::{AuthError, TodoApiError},
    },
    models::user_model::SlimUser,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub email: String,
    pub id: String,
    #[serde(with = "ts_seconds")]
    pub exp: DateTime<Utc>,
}

impl<'a> Claimable<'a> for Claims {}

impl From<SlimUser> for Claims {
    fn from(user: SlimUser) -> Self {
        use std::ops::Add;
        Claims {
            email: user.email,
            id: user.id.to_string(),
            exp: Utc::now().add(chrono::Duration::days(1)),
        }
    }
}

impl From<&SlimUser> for Claims {
    fn from(user: &SlimUser) -> Self {
        use std::ops::Add;

        let u = user.clone();

        Claims {
            email: u.email.clone(),
            id: u.id.to_string(),
            exp: Utc::now().add(chrono::Duration::days(1)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DecodedUser {
    pub email: String,
    pub id: String,
}

impl From<Claims> for DecodedUser {
    fn from(c: Claims) -> Self {
        DecodedUser {
            email: c.email,
            id: c.id,
        }
    }
}

pub struct Authenticated(DecodedUser);

/// Implementing `FromRequest` allows to extract `DecodedUser`
/// from any incoming request where `BasicAuth` middleware is used
impl FromRequest for Authenticated {
    type Error = TodoApiError;
    // Using `Ready` Future as we don't do any
    // async operation in the `from_request` function
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        // Get cloned value of decoded user from request
        let value = req.extensions().get::<DecodedUser>().cloned();

        let result = match value {
            Some(v) => Ok(Authenticated(v)),
            None => Err(TodoApiError::AuthError(AuthError::InvalidToken)),
        };

        futures::future::ready(result)
    }
}

/// Implement deref for `Authenticated` to
/// directly refer to `DecodedUser` when using `.` notation for `Authenticated`
/// Example
/// ```rust
///  let a = Authenticated(DecodedUser{id: "".to_string(), email: String::new()});
///
///  assert_eq!(a.email, String::new()); // refering to `DecodedUser` email directly
/// ```
impl std::ops::Deref for Authenticated {
    type Target = DecodedUser;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct BasicAuth;

pub struct AuthMiddleware<S> {
    service: S,
}

/// Implement `Transform` for Convert `BasicAuth` struct to `AuthMiddleware`
impl<S, B> Transform<S, ServiceRequest> for BasicAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;

    type Error = actix_web::Error;

    type InitError = ();

    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddleware { service })
    }
}

/// Implement Service
impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;

    type Error = actix_web::Error;

    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let error: TodoApiError;

        match req.headers().get("Authorization") {
            Some(auth_header) => match decode_token::<Claims>(auth_header) {
                Ok(token) => {
                    {
                        let mut extensions = req.extensions_mut();

                        extensions.insert::<DecodedUser>(token.into());
                    }

                    // Return to next middleware/handler on appending to extensions
                    return Box::pin(
                        self.service
                            .call(req)
                            .map(|res| res.map(|res| res.map_into_left_body())),
                    );
                }
                Err(err) => {
                    error = TodoApiError::AuthError(err);
                }
            },
            None => {
                error = TodoApiError::AuthError(AuthError::NoAuthorizationHeader);
            }
        }

        return Box::pin(ready(Ok(
            req.into_response(error.to_response().map_into_right_body())
        )));
    }
}
