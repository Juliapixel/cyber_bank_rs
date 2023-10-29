use actix_web::{HttpRequest, Responder, web::Json, HttpResponse};
use serde::{Serialize, Deserialize};
use sqlx::PgPool;

#[derive(Serialize, Deserialize)]
pub struct LoginRequester {
    username: String,
    password: String
}

pub async fn login(req: HttpRequest, userinfo: Json<LoginRequester>) -> impl Responder {
    let selection = sqlx::query_as!(
        super::DbUser,
        "SELECT * FROM users WHERE username = $1;",
        userinfo.username
    ).fetch_one(req.app_data::<PgPool>().unwrap()).await;
    match selection {
        Ok(o) => {
            let hashed_salted_passwd = super::salt_and_hash(userinfo.password.clone(), o.salt.as_slice());
            if o.password == hashed_salted_passwd { return HttpResponse::Ok() } else { return HttpResponse::Forbidden() }
        },
        Err(_) => return HttpResponse::Forbidden(),
    };
}
