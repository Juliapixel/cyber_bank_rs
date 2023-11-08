/// implementation for generating the JWT tokens using the creat [jsonwebtoken](jsonwebtoken)
pub mod jwt;

/// includes the customizable [middleware](actix_web::middleware) used for
/// validating JWT tokens and their scopes for endpoints, making it easy to
/// protect certain routes
pub mod middleware;
pub use middleware::ScopeValidator;

#[allow(unused_imports)]
mod tests {
    use actix_web::{test, App, web, dev::Service, http::StatusCode};
    use chrono::Utc;

    use super::{ScopeValidator, jwt::JwtClaims};

    #[test]
    async fn test_middleware() {
        let app = test::init_service(
            App::new()
                .wrap(ScopeValidator::new(&[]))
                .service(web::resource("/test").to(|| async { "OK" }))
        ).await;

        let test_valid = test::TestRequest::with_uri("/test")
            .insert_header((
                "Authorization",
                format!("Bearer {}", JwtClaims::new(Vec::new(), uuid::Builder::nil().into_uuid(), chrono::Utc::now() + chrono::Days::new(1)).generate_token())
            )).to_request();

        let res_valid = app.call(test_valid).await.unwrap();

        assert_eq!(res_valid.status(), StatusCode::OK);

        let test_invalid = test::TestRequest::with_uri("/test").to_request();

        let res_invalid = app.call(test_invalid).await.unwrap();

        assert_eq!(res_invalid.status(), StatusCode::FORBIDDEN);
    }
}
