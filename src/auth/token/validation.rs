use std::{future::Future, pin::Pin, fmt::Display};

use actix_web::{dev::{Service, ServiceResponse, ServiceRequest}, body::{EitherBody, BoxBody}, HttpResponse};
use futures_util::TryFutureExt;
use log::debug;

use super::jwt::{Scope, TokenParsingError, decode_token};

pub struct ScopeValidatorMiddleware<S> {
    service: S,
    required: &'static [Scope]
}

#[derive(Debug)]
pub enum ScopeValidationError {
    NoToken,
    BadToken(TokenParsingError),
    ExpiredToken,
    InvalidScopes
}

impl Display for ScopeValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "{}",
            match &self {
                _ => "bad auth header",
            }
        )
    }
}

impl std::error::Error for ScopeValidationError{}

pub struct ScopeValidator {
    required: &'static [Scope]
}

impl<S, B> actix_web::dev::Transform<S, ServiceRequest> for ScopeValidator
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;

    type Error = actix_web::Error;

    type Transform = ScopeValidatorMiddleware<S>;

    type InitError = ();

    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(ScopeValidatorMiddleware {
            service: service,
            required: self.required,
        }))
    }
}

impl<S, B> Service<ServiceRequest> for ScopeValidatorMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;

    type Error = actix_web::Error;

    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, ctx: &mut core::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        use ScopeValidationError as Sve;
        let mut token_invalid = false;

        let token = if let Some(header) = req.headers().get("Authorization") {
            match header.to_str() {
                Ok(o) => {
                    debug!("auth header: {o}");
                    if let Some((id, token)) = o.split_once(" ") {
                        if id != "Bearer" {
                            Err(TokenParsingError::InvalidHeader)
                        } else {
                            Ok(token)
                        }
                    } else {
                        Err(TokenParsingError::NoIdentifier)
                    }
                },
                Err(_) => Err(TokenParsingError::InvalidHeader)
            }
        } else {
            Err(TokenParsingError::HeaderMissing)
        };

        if let Ok(token) = token {
            match decode_token(token) {
                Ok(o) => { token_invalid = !self.required.iter().all(|s| o.scope.contains(s)); },
                Err(_) => { token_invalid = true; },
            };
        } else {
            token_invalid = true;
        }

        if token_invalid {
            debug!("token used is invalid: {token:?}");
            return Box::pin(async move {
                    Ok(req.into_response(HttpResponse::Forbidden().finish()).map_into_right_body())
                }
            )
        }

        return Box::pin(self.service.call(req).map_ok(|o| o.map_into_left_body()))
    }
}

impl ScopeValidator {
    pub fn new(scopes: &'static [Scope]) -> Self {
        Self {
            required: scopes
        }
    }
}
