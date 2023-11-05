/// implementation for generating the JWT tokens using the creat [jsonwebtoken](jsonwebtoken)
pub mod jwt;

/// includes the customizable [middleware](actix_web::middleware) used for
/// validating JWT tokens and their scopes for endpoints, making it easy to
/// protect certain routes
pub mod middleware;
