use actix_web::{HttpServer, App, web, middleware::Logger};
use cyber_bank_rs::{db, auth::token::validation::ScopeValidator};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default()
            .default_filter_or("DEBUG")
    );

    let pool = db::get_db_pool().await;

    // sets up tables and stuff for the database (in case it wasn't already set up)
    db::set_up_db_tables(&pool).await;

    HttpServer::new(move || {
        let conn = pool.clone();
        App::new()
            .wrap(Logger::default())
            // authentication endpoints
            .service(
                web::scope("/auth")
                    .wrap(ScopeValidator::new(&[]))
                    .service(
                        // yeah, there's only one version, so what
                        web::scope("/v1")
                            .configure(cyber_bank_rs::auth::config)
                    )
                    // should be set to latest version
                    .configure(cyber_bank_rs::auth::config)
            )
            .app_data(conn)
    }).bind(("0.0.0.0", 8080))
        .unwrap()
        .run()
        .await
}
