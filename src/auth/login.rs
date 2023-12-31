use actix_web::{HttpRequest, Responder, web::Json, HttpResponse};
use serde::{Serialize, Deserialize};
use sqlx::PgPool;

use super::token::jwt::{JwtClaims, Scope};

#[derive(Serialize, Deserialize)]
pub struct LoginRequester {
    username: String,
    password: String
}

#[derive(Serialize, Deserialize)]
struct ValidLoginResponse {
    token: String
}

/// checks that given credentials are valid and returns a
/// scoped authorization token that allows users to perform common tasks
pub async fn login(req: HttpRequest, userinfo: Json<LoginRequester>) -> impl Responder {
    let selection = sqlx::query_as!(
        super::DbUser,
        "SELECT * FROM users WHERE username = $1;",
        userinfo.username
    ).fetch_one(req.app_data::<PgPool>().unwrap()).await;
    match selection {
        Ok(o) => {
            let hashed_salted_passwd = super::salt_and_hash(userinfo.password.clone(), o.salt.as_slice());
            if o.password == hashed_salted_passwd {
                return HttpResponse::Ok()
                    .json(ValidLoginResponse {
                        token: JwtClaims::new(
                            Scope::USER_LOGIN.to_vec(),
                            o.user_id,
                            chrono::Utc::now() + chrono::Days::new(30)
                        ).generate_token()
                    });
            } else {
                return HttpResponse::Forbidden().finish();
            }
        },
        Err(_) => return HttpResponse::Forbidden().respond_to(&req),
    };
}
