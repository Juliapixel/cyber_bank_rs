use std::{sync::OnceLock, fmt::Display, future::{Ready, ready}};

use actix_web::{FromRequest, ResponseError};
use chrono::Utc;
use jsonwebtoken::{EncodingKey, DecodingKey, Validation, Algorithm};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(rename_all="snake_case")]
pub enum Scope {
    User,
    UserInfo,
}

impl Scope {
    pub const USER_LOGIN: &[Self] = &[
        Self::User,
        Self::UserInfo
    ];
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(deny_unknown_fields)]
pub struct JwtClaims {
    iat: chrono::DateTime<Utc>,
    sub: String,
    exp: chrono::DateTime<Utc>,
    pub scope: Vec<Scope>
}

impl JwtClaims {
    pub fn new(scope: Vec<Scope>, subject: String, expiration: chrono::DateTime<Utc>) -> Self {
        Self {
            iat: chrono::Utc::now(),
            sub: subject,
            exp: expiration,
            scope: scope
        }
    }
}

static SECRET: OnceLock<Vec<u8>> = OnceLock::new();

fn get_secret() -> &'static [u8] {
    SECRET.get_or_init(|| {
        #[cfg(debug_assertions)]
        {
            b"debug".to_vec()
        }
        #[cfg(not(debug_assertions))]
        {
            todo!("find a way to safely get a secret")
        }
    })
}

pub fn generate_token(claims: &JwtClaims) -> String {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(Algorithm::HS256),
        claims,
        &EncodingKey::from_secret(get_secret())
    ).unwrap()
}

pub fn decode_token(token: &str) -> Result<JwtClaims, jsonwebtoken::errors::Error> {
    match jsonwebtoken::decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(get_secret()),
        &Validation::new(Algorithm::HS256)
    ) {
        Ok(o) => Ok(o.claims),
        Err(e) => Err(e)
    }
}

pub struct TokenHeader(String);

#[derive(Debug)]
pub enum TokenParsingError {
    NoIdentifier,
    HeaderMissing,
    InvalidHeader
}

impl Display for TokenParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match &self {
                Self::HeaderMissing => "Missing Authorization Header",
                _ => "Header Malformed"
            }
        )
    }
}

impl ResponseError for TokenParsingError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::FORBIDDEN
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let res = actix_web::HttpResponse::new(self.status_code());

        let buf = format!("{}", self);

        res.set_body(actix_web::body::BoxBody::new(buf))
    }
}

impl FromRequest for TokenHeader {
    type Error = TokenParsingError;

    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        if let Some(header) = req.headers().get("Authorization") {
            match header.to_str() {
                Ok(o) => {
                    if let Some((id, token)) = o.split_once(" ") {
                        if id != "Bearer" {
                            return ready(Err(TokenParsingError::InvalidHeader))
                        }
                        return ready(Ok(Self(token.to_string())))
                    } else {
                        return ready(Err(TokenParsingError::NoIdentifier))
                    }
                },
                Err(_) => return ready(Err(TokenParsingError::InvalidHeader)),
            }
        } else {
            return ready(Err(TokenParsingError::HeaderMissing))
        }
    }
}
